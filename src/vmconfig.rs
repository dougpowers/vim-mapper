// Copyright 2022 Doug Powers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{collections::HashMap, fs};

use druid::{Color};
use serde::{Serialize, Deserialize};

use crate::constants::{DEFAULT_CONFIG_DIR_NAME, DEFAULT_CONFIG_FILE_NAME};

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum ColorScheme {
    LIGHT,
    DARK,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VMConfig {
    pub color_scheme: ColorScheme,
    pub dark_palette: HashMap<String, (u8,u8,u8,u8)>,
    pub light_palette: HashMap<String, (u8,u8,u8,u8)>,
}

impl Default for VMConfig {

    fn default() -> Self {
        let mut dark_palette: HashMap<String, (u8,u8,u8,u8)> = HashMap::new();
        let mut light_palette: HashMap<String, (u8,u8,u8,u8)> = HashMap::new();
        light_palette.insert("label-text-color".to_string(), (0,0,0,255));
        light_palette.insert("node-border-color".to_string(), (0,0,0,255));
        light_palette.insert("active-node-border-color".to_string(), (125,125,255,255));
        light_palette.insert("target-node-border-color".to_string(), (255,125,125,255));
        light_palette.insert("node-background-color".to_string(), (200,200,200,255));
        light_palette.insert("edge-color".to_string(), (192,192,192,255));
        light_palette.insert("compose-indicator-text-color".to_string(), (255,0,0,255));
        light_palette.insert("sheet-background-color".to_string(), (255,255,255,255));
        dark_palette.insert("label-text-color".to_string(), (255,255,255,255));
        dark_palette.insert("node-border-color".to_string(), (215,215,215,255));
        dark_palette.insert("active-node-border-color".to_string(), (125,125,255,255));
        dark_palette.insert("target-node-border-color".to_string(), (255,125,125,255));
        dark_palette.insert("node-background-color".to_string(), (100,100,100,255));
        dark_palette.insert("edge-color".to_string(), (132,132,132,255));
        dark_palette.insert("compose-indicator-text-color".to_string(), (255,0,0,255));
        dark_palette.insert("sheet-background-color".to_string(), (0,0,0,255));
        VMConfig {
            color_scheme: ColorScheme::LIGHT,
            light_palette,
            dark_palette,
        }
    }

}

impl VMConfig {
    pub fn load() -> Result<Self, String> {
        if let Some(mut path) = dirs::config_dir() {
            path.push(DEFAULT_CONFIG_DIR_NAME);
            if !path.clone().exists() {
                if let Ok(_) = fs::create_dir(path.clone()) {
                } else {
                    return Err(format!("Couldn't create configuration directory at {:?}", path.clone()));
                }
            }
            path.push(DEFAULT_CONFIG_FILE_NAME);
            if !path.exists() {
                println!("No config file found. Creating at {}", path.display());
                let mut config = VMConfig::default();
                let system_mode = dark_light::detect();

                match system_mode {
                    dark_light::Mode::Light => {
                        config.set_color_scheme(ColorScheme::LIGHT);
                    }
                    dark_light::Mode::Dark => {
                        config.set_color_scheme(ColorScheme::DARK);
                    }
                }

                fs::write(path, serde_json::to_string(&config).ok().expect("Failed to serialize default config!")).expect("Failed to write default config to file");
                return Ok(config)
            } else {
                if let Ok(string) = fs::read_to_string(path.clone()) {
                    if let Ok(config) = serde_json::from_str::<VMConfig>(&string) {
                        return Ok(config)
                    } else {
                        return Err(format!("Couldn't serialize config file at {}", path.display()))
                    }
                }
            }
        }
        Err("General filesystem error".to_string())
    }

    pub fn save(&self) -> Result<String, String> {
        let mut path = dirs::config_dir().expect("no user config dir found");
        path.push(DEFAULT_CONFIG_DIR_NAME);
        path.push(DEFAULT_CONFIG_FILE_NAME);
        fs::write(path, serde_json::to_string_pretty(self).ok().expect("Failed to serialize config")).ok().expect("Failed to save config");
        Ok("".to_string())
    }

    pub fn set_color_scheme(&mut self, scheme: ColorScheme) {
        self.color_scheme = scheme;
    }

    #[allow(dead_code)]
    pub fn get_color_scheme(&self) -> ColorScheme {
        self.color_scheme.clone()
    }

    pub fn toggle_color_scheme(&mut self) {
        match self.color_scheme {
            ColorScheme::LIGHT => {
                self.color_scheme = ColorScheme::DARK;
            }
            ColorScheme::DARK => {
                self.color_scheme = ColorScheme::LIGHT;
            }
        }
    }

    pub fn get_color(&self, key: String) -> Result<Color, String> {
        match self.color_scheme {
            ColorScheme::LIGHT => {
                if let Some((r,g,b,a)) = self.light_palette.get(&key.clone()) {
                    return Ok(Color::rgba8(*r,*g,*b,*a));
                } else {
                    Err("Color not found.".to_string())
                }
            }
            ColorScheme::DARK => {
                if let Some((r,g,b,a)) = self.dark_palette.get(&key.clone()) {
                    return Ok(Color::rgba8(*r,*g,*b,*a));
                } else {
                    Err("Color not found.".to_string())
                }
            }
        }
    }
}