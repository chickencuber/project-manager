use easy_menu::{Event, Key, Menu, MenuOptions, Style};

use core::str;
use std::{collections::HashMap, env::{self, args, current_exe}, fs::{create_dir, read_dir, read_to_string, remove_dir_all, write}, os::unix::{fs::PermissionsExt, process::CommandExt}, path::{Path, PathBuf}, process::Command};

use serde::{Deserialize, Serialize};

use ron::{de::from_str, ser::to_string_pretty};

#[derive(Deserialize, Serialize, Debug)]
struct _Data {
    pub editor: String,
    pub last: String,
    pub libraries: String,
    pub categories: HashMap<String, Category>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Category {
    pub supported_types: Vec<String>,
    pub parent_dir: String,
}

#[derive(Debug)]
struct Data {
    pub data: _Data,
    pub project_types: Vec<String>,
}


fn xdg_config_home() -> PathBuf {
    if let Ok(path) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(path);
    } else {
        let mut p = PathBuf::from(std::env::var("HOME").unwrap());
        p.push(".config");
        return p;
    }
}

impl Data {
    fn read() -> Self {
        let mut path = xdg_config_home();
        path.push("prmn");
        path.push("data.ron");
        let mut folder = path.clone();
        let text = read_to_string(path).unwrap();
        folder.pop();
        folder.push("types");
        let types = read_dir(folder).unwrap();
        let mut project_types = Vec::new();
        for v in types {
            let file = v.unwrap();
            let mut name = file.file_name().into_string().unwrap();
            let permissions = file.metadata().unwrap().permissions();
            let is_executable = permissions.mode() & 0o111 != 0;
            if !is_executable {
                continue;
            }
            if !name.ends_with(".sh") {
                continue;
            }
            name.pop();
            name.pop();
            name.pop();
            project_types.push(name); 
        }
        return Self {
            data: from_str(text.as_str()).unwrap(),
            project_types,
        };
    }
    fn pretty(&self) -> String {
        return to_string_pretty(&self.data, ron::ser::PrettyConfig::default()).unwrap()
    }
    fn write(&self) {
        let mut path = current_exe().unwrap();
        path.pop();
        path.pop();
        path.pop();
        path.push("data.ron");
        write(path, self.pretty()).unwrap();
    }
}

fn render_projects(category: &String, data: &Data) -> String {
    let mut items = vec!["<..>".to_string()];
    items.append(&mut get_all_files(&data, Some(category)));
    if items.len() == 0 {
        items.push("<None>".to_string());
    }
    let mut menu = Menu::new(items, MenuOptions {
        key: |key, menu| {
            match key {
                Key::ArrowDown => Event::Down,
                Key::Char('j') => Event::Down,
                Key::ArrowUp => Event::Up,
                Key::Char('k') => Event::Up,
                Key::Escape => Event::Return("<..>".to_string()),
                Key::Char('q')=> Event::Return("<..>".to_string()),
                Key::Char('f') => Event::Return("<Fuzzy-Finder>".to_string()),
                Key::Enter => {
                    if menu.options[menu.selected] == "<None>" {
                        return Event::None;
                    }
                    Event::Select
                },
                Key::Char('d') => {
                    if menu.options[menu.selected] == "<..>" {
                       return Event::None; 
                    }
                    if menu.options[menu.selected] == "<None>" {
                        return Event::None;
                    }
                    if Menu::prompt("are you sure? [y/n]") == "y" {
                        let mut data = "<Delete>".to_string();
                        data.push_str(&menu.options[menu.selected]);
                        return Event::Return(data);
                    } else {
                        return Event::None;
                    }
                },
                Key::Char('a') => Event::Return("<Create>".to_string()),
                _ => Event::None,
            }
        },
        style_selected: Style::new().on_color256(0),
        ..Default::default()
    });
    let mut d = menu.run();
    if d.starts_with("<Delete>") {
        d.remove(0);
        d.remove(0);
        d.remove(0);
        d.remove(0);
        d.remove(0);
        d.remove(0);
        d.remove(0);
        d.remove(0);
        let mut path = data.data.categories[category].parent_dir.clone();
        path.push_str(&d);
        remove_dir_all(path).unwrap();
        d = "<Delete>".to_string();
    }
    return d;
}

fn get_all_files(data: &Data, folder: Option<&String>) -> Vec<String> {
    if let Some(category) = folder {
        let mut items = vec![]; 
        let path = data.data.categories[category].parent_dir.clone();
        for v in read_dir(path).unwrap() {
            let file = v.unwrap();
            if !file.metadata().unwrap().is_dir() {
                continue;
            }
            items.push(file.file_name().into_string().unwrap());
        } 
        items.sort();
        return items;
    } else {
        let mut items = vec![];
        for (k, v) in data.data.categories.iter() {
            for v in read_dir(v.parent_dir.clone()).unwrap() {
                let file = v.unwrap();
                if !file.metadata().unwrap().is_dir() {
                    continue;
                }
                let mut path = k.clone();
                path.push('/');
                let name = file.file_name().into_string().unwrap();
                path.push_str(&name);
                items.push(path);
            }
        }
        items.sort();
        return items;
    }
}

fn fzf(input: Vec<String>) -> Option<String>{
    let mut path = current_exe().unwrap();
    path.pop();
    path.pop();
    path.pop();
    path.push("run.sh");
    let output = Command::new(path).arg(input.join("\\n")).output().unwrap();
    if output.status.success() {
        let stdout = str::from_utf8(&output.stdout).unwrap();
        return Some(stdout.to_string());
    } else {
        return None;
    }
}

fn get_types(data: &Data, category: &String) -> Vec<String> {
    let temp = data.project_types.clone();
    let mut f = vec![];
    for v in temp {
        if data.data.categories[category].supported_types.contains(&v) {
            f.push(v);
        }
    }
    f.sort();
    return f;
}

fn create_project(data: &Data, category: &String) {
    let types = get_types(data, category);
    let mut out;
    if types.len() != 1 {
        let mut menu = Menu::new(types.clone(), MenuOptions {
            key: |key, _| {
                match key {
                    Key::ArrowDown => Event::Down,
                    Key::Char('j') => Event::Down,
                    Key::ArrowUp => Event::Up,
                    Key::Char('k') => Event::Up,
                    Key::Escape => Event::Return("<Canceled>".to_string()),
                    Key::Char('q') => Event::Return("<Canceled>".to_string()),
                    Key::Char('f') => {
                        return Event::Return("<Fuzzy-Finder>".to_string());
                    }
                    Key::Enter => {
                        Event::Select
                    },
                    _ => Event::None,
                }
            },
            style_selected: Style::new().on_color256(0),
            ..Default::default()
        });
        out = menu.run();
        if out == "<Canceled>" {
            return;
        }
        if out == "<Fuzzy-Finder>" {
            let o = fzf(types);
            if let Some(v) = o {
                out = v;
            } else {
                return;
            }
        }
    } else {
        out = types[0].clone();
    }

    out.push_str(".sh");
    let mut path = data.data.categories[category].parent_dir.clone();
    path.push_str(&Menu::prompt("name"));
    create_dir(&path).unwrap();
    let mut sh = current_exe().unwrap();
    sh.pop();
    sh.pop();
    sh.pop();
    sh.push("types");
    sh.push(out);
    println!("{sh:?}");
    Command::new(sh).current_dir(path).spawn().unwrap();
}

fn main() {
    let args: Vec<String> = args().collect();
    let val = args.get(1);
    let mut find = false;
    if let Some(v) = val {
        let data = Data::read();
        if v.to_lowercase() == "last" || v.to_lowercase() == "-last" || v.to_lowercase() == "l" || v.to_lowercase() == "-l" {
            Command::new(data.data.editor).arg(data.data.last).exec();
            return;
        }
        find = v.to_lowercase() == "f" || v.to_lowercase() == "-f" || v.to_lowercase() == "find" || v.to_lowercase() == "-find";
        if v.to_lowercase() == "lib" {
            if let Some(lib) = args.get(2) {
                let mut path = Path::new(&data.data.libraries).to_path_buf();
                path.push(lib);
                if path.exists() {
                    if path.is_dir() {
                        if !Path::new("./libraries").exists() {
                            create_dir("./libraries").unwrap(); 
                        } 
                        Command::new("cp")
                            .arg("-r")
                            .arg(&path)
                            .arg(format!("./libraries/{}", lib))
                            .spawn().unwrap();
                        path.push("depends");
                        if path.exists() {
                            if path.is_file() {
                                let str: Vec<String> = read_to_string(&path).unwrap().lines().map(String::from).collect();
                                for lib in str {
                                    if lib.trim() == "" {
                                        continue;
                                    }
                                    let name = format!("./libraries/{}", lib);
                                    let path =  Path::new(&name);
                                    if path.exists() {
                                        continue;
                                    }
                                    Command::new(current_exe().unwrap())
                                        .arg("lib")
                                        .arg(lib.trim()).spawn().unwrap();
                                    }
                            }
                        }
                        return;    
                    }
                }
                println!("not valid library");
                return;
            } else {
                for c in Path::new(&data.data.libraries).read_dir().unwrap() {
                    println!("{}", c.unwrap().file_name().to_str().unwrap());
                }
                return;
            }
        }
    }
    loop {
        let mut data = Data::read();

        let mut items = vec![];
        if data.data.categories.len() == 0 {
            items.push("<None>".to_string());
        } else {
            for k in data.data.categories.keys() {
                items.push(k.to_string());
            } 
            items.sort();
        }
        let mut menu = Menu::new(items, MenuOptions {
            key: |key, menu| {
                match key {
                    Key::ArrowDown => Event::Down,
                    Key::Char('j') => Event::Down,
                    Key::ArrowUp => Event::Up,
                    Key::Char('k') => Event::Up,
                    Key::Escape => Event::Return("<Canceled>".to_string()),
                    Key::Char('q') => Event::Return("<Canceled>".to_string()),
                    Key::Char('f') => {
                        return Event::Return("<Fuzzy-Finder>".to_string());
                    }
                    Key::Enter => {
                        if menu.options[menu.selected] == "<None>" {
                            return Event::None;
                        }
                        Event::Select
                    },
                    _ => Event::None,
                }
            },
            style_selected: Style::new().on_color256(0),
            ..Default::default()
        });
        let folder;
        if find {
            find = false;
            folder = "<Fuzzy-Finder>".to_string();
        } else {
            folder=menu.run();
        }
        if folder == "<Canceled>" {
            break;
        }
        if folder == "<Fuzzy-Finder>" {
            let output = fzf(get_all_files(&data, None));
            if let Some(stdout) = output {
                let out: Vec<String> = stdout.split("/").map(|s| s.to_string()).collect();
                let mut path = data.data.categories[&out[0]].parent_dir.clone();
                path.push_str(&out[1]);       
                let mut str = path.to_string();
                str.pop();
                data.data.last = str.to_string();
                data.write();
                Command::new(data.data.editor).arg(&str).exec();
            } else {
                continue;
            }
            break;
        }
        let mut file = render_projects(&folder, &data);
        if file == "<..>" {
            continue;
        }
        if file == "<Fuzzy-Finder>" {
            let out = fzf(get_all_files(&data, Some(&folder)));
            if let Some(v) = out {
                file = v;
                file.pop();
            } else {
                continue;
            }
        }
        if file == "<Create>" {
            create_project(&data, &folder);            
            continue;
        }
        if file == "<Delete>" {
            continue;
        }

        let mut path = data.data.categories[&folder].parent_dir.clone();
        path.push_str(&file);
        data.data.last = path.to_string();
        data.write();
        Command::new(data.data.editor).arg(&path).exec();
        break;
    }
}
