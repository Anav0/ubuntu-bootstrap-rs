use colored::*;
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::LineWriter;
use std::process::Command;

#[allow(dead_code, unused_variables, unused_imports)]
fn main() {
    let home_dir = env::var("HOME").expect("Failed to fetch HOME env variable");

    update_apt();

    install_apps();

    //Updates zshrc and bashrc files with exports defined in 'export' file
    handle_exports(home_dir);
}

fn update_apt() {
    println!("{}", "Updating apt".bold());
    let output = Command::new("apt")
        .arg("update")
        .output()
        .expect("Failed to update apt");
    let err = String::from_utf8_lossy(&output.stderr);
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("{}", err);
}
fn install_apps() {
    let mut apps_file = String::new();
    File::open("./apt_apps")
        .expect("Failed to load apps file")
        .read_to_string(&mut apps_file)
        .expect("Failed to load apps file");

    let names: Vec<&str> = apps_file.split("\n").collect();

    let output = Command::new("apt")
        .arg("install")
        .arg("-y")
        .args(names)
        .output()
        .expect("Failed to install apt apps");

    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("{}", String::from_utf8_lossy(&output.stderr));
}
fn handle_exports(home_dir: String) {
    let mut exports_found_in_bashrc: HashSet<String> = HashSet::new();
    let mut exports_found_in_zshrc: HashSet<String> = HashSet::new();

    println!("{}\n", "Inserting exports into zshrc and bashrc".bold());

    for (path, exports_found) in [
        (".zshrc", &mut exports_found_in_zshrc),
        (".bashrc", &mut exports_found_in_bashrc),
    ] {
        let f = OpenOptions::new()
            .read(true)
            .open(&format!("{}/{}", home_dir, path))
            .expect(&format!("Failed to open '{}'", path));

        let reader = BufReader::new(f);

        for line in reader.lines() {
            let content = line.expect(&format!("Failed to read line of '{}'", path));
            if content.starts_with("export ") {
                exports_found.insert(content);
            }
        }
    }

    let mut bashrc_appender = OpenOptions::new()
        .append(true)
        .open(format!("{}/{}", home_dir, ".bashrc"))
        .expect("Failed to open zshrc");

    let mut zshrc_appender = OpenOptions::new()
        .append(true)
        .open(format!("{}/{}", home_dir, ".zshrc"))
        .expect("Failed to open zshrc");

    for (path, writer, exports_found) in [
        (".zshrc", &mut zshrc_appender, &exports_found_in_zshrc),
        (".bashrc", &mut bashrc_appender, &exports_found_in_bashrc),
    ] {
        let exports_file = File::open("./exports").expect("Failed to open 'exports' file");
        let exports_reader = BufReader::new(exports_file);
        for line in exports_reader.lines() {
            let content = line.expect("Failed to process line in 'export' file ");
            if content.starts_with("export ") {
                if exports_found.contains(&content) {
                    println!("{} {}", path.yellow().italic(), content.yellow());
                } else {
                    println!("{} {}", path.green().italic(), &content.green());
                    writer.write((content + "\n").as_bytes()).expect("msg");
                }
            }
        }
    }
}
