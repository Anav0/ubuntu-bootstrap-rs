use colored::*;
use core::panic;
use std::collections::HashSet;
use std::env::{args, current_exe, temp_dir, var, var_os};
use std::fs::{copy, create_dir_all, read_dir, remove_dir_all, DirBuilder, File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;
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
        print_header("Installing apt apps");
        let mut apps_file = String::new();
        File::open(self.path_to_app_names)
            .expect("Failed to load apps file")
            .read_to_string(&mut apps_file)
            .expect("Failed to load apps file");

        let names: Vec<&str> = apps_file.split("\n").collect();

        print!("{}", "Installing apt apps: ".bright_white());
        let mut i = 1;
        let total = names.len();
        for name in names {
            if name.trim() == "" {
                continue;
            }
            print!("[{}/{}] Installing {}", i, total, name);
            let output = Command::new("sudo")
                .arg("apt")
                .arg("install")
                .arg("-y")
                .arg(name.trim())
                .output()
                .expect(&format!("Failed to install: {}", name));

            println!("{}", String::from_utf8_lossy(&output.stderr));
            if !output.status.success() {
                panic!("Failed to install apt apps")
            }
            i += 1;
        }

        Ok(String::from("Apt apps installed!"))
    }
}

struct CargoInstaller<'a> {
    path_to_app_names: &'a str,
}

impl<'a> CargoInstaller<'a> {
    fn new(path_to_app_names: &'a str) -> Self {
        Self { path_to_app_names }
    }
}

impl AppsInstaller for CargoInstaller<'_> {
    fn install(&self) -> Result<String, String> {
        print_header("Installing cargo apps");
        let mut apps_file = String::new();
        File::open(self.path_to_app_names)
            .expect("Failed to load cargo apps file")
            .read_to_string(&mut apps_file)
            .expect("Failed to load cargo apps file");

        let names: Vec<&str> = apps_file.split("\n").collect();
        print!("{}", "Installing cargo apps: ".bright_white());
        for name in &names {
            print!("{} ", name.bright_white());
        }

        for name in names {
            if name.trim() == "" {
                continue;
            }
            let output = Command::new("cargo")
                .arg("install")
                .arg(name.trim())
                .output()
                .expect(&format!("Failed to install '{}'", name.trim()));

            println!("{}", String::from_utf8_lossy(&output.stderr));
            if !output.status.success() {
                panic!("Failed to install cargo apps")
            }
            println!("{} {}", "Installed".green(), name.green());
        }
        Ok(String::from("Cargo apps installed"))
    }
}

struct ZshInstaller;
impl AppsInstaller for ZshInstaller {
    fn install(&self) -> Result<String, String> {
        print_header("Installing oh my zsh");
        let output =
        Command::new("sh")
        .arg("c")
        .arg("$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)")
        .output().expect("Failed to install oh my zsh");

        println!("{}", String::from_utf8_lossy(&output.stderr));

        Ok(String::from("Installed oh my zsh"))
    }
}

#[allow(dead_code, unused_variables, unused_imports)]
fn main() {
    let home_dir = var("HOME")
        .expect("Failed to read $HOME variable")
        .to_string();

    let mut tmp_dir = temp_dir();
    let path_to_exe = current_exe().expect("Failed to get name of current executable");

    let program_name = path_to_exe
        .file_name()
        .expect("Failed to extract executable name");

    tmp_dir.push(program_name);
    let tmp_dir_str = tmp_dir.to_str().unwrap();

    println!(
        "{}\n",
        "Setting up ubuntu system...".bold().bright_magenta()
    );

    update_apt();

    install_apps();

    //Install fonts?

    place_dotfiles(&home_dir, tmp_dir_str);

    place_exports(&home_dir);

    println!("\n{}\n", "Done!".bold().bright_magenta());
}

fn print_header(text: &str) {
    println!("{}", text.bold().bright_blue());
}

fn copy_directory(path: &str, tmp_dir: &str, home_dir: &str) {
    let folder_iter = read_dir(path).expect("Failed to read files in tmp folder for dotfiles");
    for read_dir in folder_iter {
        let entry = read_dir.expect(format!("Failed to read file {:?} ", path).as_str());
        let entry_path = entry.path();
        let entry_path_str = entry_path.to_str().unwrap();

        if entry_path_str.contains(".git") {
            continue;
        }
        if entry.metadata().unwrap().is_dir() {
            let dst_path_str = entry_path_str.replace(tmp_dir, home_dir);
            create_dir_all(&dst_path_str)
                .expect(&format!("Failed to create dir: {}", &dst_path_str));
            println!("Crated directory: {}", &dst_path_str);
            copy_directory(entry_path_str, tmp_dir, home_dir);
        } else {
            let dst_path_str = entry_path_str.replace(tmp_dir, home_dir);
            copy(entry_path_str, &dst_path_str).expect(&format!(
                "Failed to copy file from {} to {}",
                entry_path_str, &dst_path_str
            ));
            println!("Copied {} to {}", &entry_path_str, &dst_path_str);
        }
    }
}

fn place_dotfiles(home_dir: &str, tmp_folder: &str) {
    print_header("Cloning and placing dotfiles");

    DirBuilder::new()
        .create(tmp_folder)
        .expect("Failed to create tmp folder for dotfiles");

    let output = Command::new("git")
        .arg("clone")
        .arg("https://github.com/Anav0/dotfiles")
        .arg(tmp_folder)
        .output()
        .expect("Failed to git clone dotfiles");

    println!("{}", String::from_utf8_lossy(&output.stderr));
    copy_directory(tmp_folder, tmp_folder, home_dir);
}

fn update_apt() {
    print_header("Updating apt");
    let status = Command::new("sudo")
        .arg("apt")
        .arg("update")
        .status()
        .expect("Failed to update apt");
    println!("{}", status);
}

fn install_apps() {
    print_header("Installing programs");
    let apt_installer = AptInstaller::new("./apt_apps");
    let zsh_installer = ZshInstaller {};
    let cargo_installer = CargoInstaller::new("./cargo_apps");
    let installers: [&dyn AppsInstaller; 3] = [&apt_installer, &zsh_installer, &cargo_installer];

    for installer in installers {
        match installer.install() {
            Err(err_msg) => println!("{}", err_msg),
            _ => {}
        }
    }
}

fn place_exports(home_dir: &str) {
    print_header("Inserting exports into zshrc and bashrc");

    let mut exports_found_in_bashrc: HashSet<String> = HashSet::new();
    let mut exports_found_in_zshrc: HashSet<String> = HashSet::new();

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
