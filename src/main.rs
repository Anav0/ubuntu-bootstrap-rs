use colored::*;
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs::copy;
use std::fs::read_dir;
use std::fs::remove_dir_all;
use std::fs::DirBuilder;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::LineWriter;
use std::path::Path;
use std::process::Command;

trait AppsInstaller {
    fn install(&self) -> Result<String, String>;
}

struct AptInstaller<'a> {
    path_to_app_names: &'a str,
}

impl<'a> AptInstaller<'a> {
    fn new(path_to_app_names: &'a str) -> Self {
        Self { path_to_app_names }
    }
}

impl AppsInstaller for AptInstaller<'_> {
    fn install(&self) -> Result<String, String> {
        let mut apps_file = String::new();
        File::open(self.path_to_app_names)
            .expect("Failed to load apps file")
            .read_to_string(&mut apps_file)
            .expect("Failed to load apps file");

        let names: Vec<&str> = apps_file.split("\n").collect();

        println!("Installing apps: {:?}", names);

        let output = Command::new("apt")
            .arg("install")
            .arg("-y")
            .args(names)
            .output()
            .expect("Failed to install apt apps");

        println!("{}", String::from_utf8_lossy(&output.stderr));

        Ok(String::from("Apt apps installed!"))
    }
}

#[allow(dead_code, unused_variables, unused_imports)]
fn main() {
    let home_dir = String::from("/home/igor");

    // update_apt();

    let apt_installer = AptInstaller::new("./apps");

    for installer in [apt_installer] {
        match installer.install() {
            Ok(msg) => println!("{}", msg),
            Err(err_msg) => println!("{}", err_msg),
        }
    }

    // place_dotfiles(&home_dir);

    // handle_exports(home_dir);

    println!("Done!");
}

fn install_apps_from_source() {}

fn copy_directory(path: &str, tmp_dir: &str, home_dir: &str) {
    let folder_iter = read_dir(path).expect("Failed to read files in tmp folder for dotfiles");
    for read_dir in folder_iter {
        let entry = read_dir.expect("msg");
        let entry_path = entry.path();
        let entry_path_str = entry_path.to_str().unwrap();
        if entry.metadata().unwrap().is_dir() {
            copy_directory(entry_path_str, tmp_dir, home_dir);
        } else {
            let dst_path_str = entry_path_str.replace(tmp_dir, home_dir);
            if dst_path_str.contains(".git") {
                continue;
            }
            copy(entry_path_str, dst_path_str)
                .expect(&format!("Failed to copy file: {}", entry_path_str));
        }
    }
}

fn place_dotfiles(home_dir: &str) {
    println!("{}", "Cloning and placing dotfiles".bold());
    let tmp_folder = Path::new("/tmp/dot");
    if tmp_folder.exists() {
        remove_dir_all(tmp_folder).expect("Failed to remove tmp folder for dotfiles");
    }

    DirBuilder::new()
        .create(tmp_folder)
        .expect("Failed to create tmp folder for dotfiles");

    let output = Command::new("git")
        .arg("clone")
        .arg("https://github.com/Anav0/dotfiles")
        .arg(tmp_folder)
        .output()
        .expect("Failed to git clone dotfiles");

    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("{}", String::from_utf8_lossy(&output.stderr));
    let tmp_folder_str = tmp_folder.to_str().unwrap();
    copy_directory(tmp_folder_str, tmp_folder_str, home_dir);
}

fn update_apt() {
    println!("{}", "Updating apt".bold());
    let output = Command::new("apt")
        .arg("update")
        .output()
        .expect("Failed to update apt");
    // println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("{}", String::from_utf8_lossy(&output.stderr));
}

fn install_apps() {}

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
