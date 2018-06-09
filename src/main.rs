extern crate reqwest;
extern crate serde_json;
extern crate rand;

use serde_json::{Value, Error};
use std::fs;
use std::str;
use std::process;
use std::io::prelude::*;
use std::fs::File;
use std::process::Command;
use std::fmt;
use rand::{thread_rng, Rng};

#[cfg(target_os = "linux")]
fn make_folders() -> String
{
    let user_c = Command::new("whoami")
        .output()
        .expect("failed to execute process");
    let user = str::from_utf8(&user_c.stdout[..]).unwrap().trim();
    fs::create_dir_all(format!("/home/{}/.interesting", user)).unwrap();
    let path = format!("/home/{}/.interesting/currednt.jpg", user);
    set_wallpaper(&path);
    return path;
}

#[cfg(target_os = "windows")]
fn make_folders() -> String
{
    let user_c = Command::new("whoami")
        .output()
        .expect("failed to execute process");
    let user = str::from_utf8(&user_c.stdout[..]).unwrap().trim();
    fs::create_dir_all(format!("C:\\Users\\{}\\.interesting", user)).unwrap();
    let path = format!("C:\\Users\\{}\\.interesting/current.jpg", user);
    set_wallpaper(&path);
    return path;
}

#[cfg(target_os = "macos")]
fn make_folders() -> String
{
    let user_c = Command::new("whoami")
        .output()
        .expect("failed to execute process");
    let user = str::from_utf8(&user_c.stdout[..]).unwrap().trim();
    fs::create_dir_all(format!("/Users/{}/.interesting", user)).unwrap();
    let path = format!("/Users/{}/.interesting/current.jpg", user);
    set_wallpaper(&path);
    return path;
}

#[cfg(target_os = "linux")]
fn set_wallpaper(path: &String)
{
    let c = Command::new("gsettings")
        .arg("set")
        .arg("org.gnome.desktop.background")
        .arg("picture-uri")
        .arg(path)
        .output()
        .expect("Failed");
}

#[cfg(target_os = "windows")]
fn set_wallpaper(path: &String)
{
    let c = Command::new("osascript")
        .arg("-e")
        .arg(format!("`tell application “Finder” to set desktop picture to POSIX file \"{path}\"`", path))
        .output()
        .expect("Failed");
}

#[cfg(target_os = "macos")]
fn set_wallpaper(path: &String)
{
    let c = Command::new("osascript")
        .arg("-e")
        .arg(format!("`tell application “Finder” to set desktop picture to POSIX file \"{path}\"`", path))
        .output()
        .expect("Failed");
}



struct Photo {
    id: String,
    owner: String,
    secret: String,
    title: String
}

struct Size {
    w: i32,
    h: i32
}

struct FlickrImage {
    url: String,
    size: Size
}

impl Photo {

    fn to_string(&mut self) -> String
    {
        let mut s: String = "".to_owned();
        s.push_str(&self.id);
        s.push_str("\n");
        s.push_str(&self.title);
        s.push_str("\n");
        s.push_str(&self.owner);
        s.push_str("\n");
        s.push_str(&self.secret);
        return s;
    }

}

impl fmt::Debug for Photo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?}, {:?})", self.id, self.title)
    }
}

fn fetch_json() -> String
{
    let url = "https://api.flickr.com/services/rest/?method=flickr.interestingness.getList&api_key=be365ea669573472ffe0f1a1cf6b3e94&per_page=50&format=json";
    let body = reqwest::get(url).unwrap().text().unwrap();
    return body[14..body.len() - 1].to_string();
}

fn json_to_structs(json: String) -> Vec<Photo>
{
    let mut photos: Vec<Photo> = Vec::new();

    let v: Value = serde_json::from_str(&json[..]).unwrap();

    let instances = v["photos"]["photo"].as_array().unwrap();

    for photo in instances {
        let p_obj = Photo {
            id: photo["id"].to_string(),
            owner: photo["owner"].to_string(),
            secret: photo["secret"].to_string(),
            title: photo["title"].to_string()
        };
        photos.push(p_obj);
    }

    return photos;
}

fn size_to_int(size: String) -> i32
{
    if size.contains("\"") {
        return size[1..size.len()-1].parse::<i32>().unwrap();
    }
    return size.parse::<i32>().unwrap();
}

fn get_original(photo: &Photo) -> FlickrImage
{
    let realId = &photo.id[1..&photo.id.len()-1];
    let url = &format!("https://api.flickr.com/services/rest/?method=flickr.photos.getSizes&api_key=be365ea669573472ffe0f1a1cf6b3e94&format=json&photo_id={}",  &realId)[..];
    let body = reqwest::get(url).unwrap().text().unwrap();
    let json = &body[14..body.len()-1];
    let v: Value = serde_json::from_str(json).unwrap();
    let sizes = v["sizes"]["size"].as_array().unwrap();
    let biggest = &sizes[sizes.len() -1];

    return FlickrImage {
      url: biggest["source"].to_string(),
        size: Size {
            w: size_to_int(biggest["width"].to_string()),
            h: size_to_int(biggest["height"].to_string())
        }
    };
}

fn save_to_machine(image: &FlickrImage, path: String)
{
    let tmp = path.clone();
    let url = &image.url[1..&image.url.len() -1];
    let mut res = reqwest::get(url).unwrap();
    let mut buf: Vec<u8> = vec![];
    res.copy_to(&mut buf).unwrap();
    let mut buffer = File::create(path).unwrap();
    buffer.write(&buf[..]);
}

fn main() {

    let path = make_folders();
    let json = fetch_json();
    let mut results = json_to_structs(json);
    rand::thread_rng().shuffle(&mut results);

    for mut photo in results {
        let orig = get_original(&photo);
        if orig.size.w >= 1920 && orig.size.h >= 1080 {
            save_to_machine(&orig, path);
            process::exit(0);
        }
    }

}
