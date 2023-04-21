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

use crate::constants::{DEFAULT_CONFIG_DIR_NAME, DEFAULT_CONFIG_FILE_NAME, CURRENT_CONFIG_FILE_VERSION};

#[allow(dead_code)]
const VERSIONS: &'static [&'static str] = &["0.4.0"];

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum ColorScheme {
    LIGHT,
    DARK,
}

#[derive(Debug,Hash,Eq,PartialEq,Clone,Serialize,Deserialize)]
pub enum VMColor {
    LabelTextColor,
    DisabledLabelTextColor,
    NodeBorderColor,
    DisabledNodeBorderColor,
    ActiveNodeBorderColor,
    TargetNodeBorderColor,
    NodeBackgroundColor,
    DisabledNodeBackgroundColor,
    EdgeColor,
    ComposeIndicatorTextColor,
    SheetBackgroundColor,
    DialogBackgroundColor,
    StackBackgroundColor,
    AlertColor,
    ButtonLight,
    ButtonDark,
    AlertButtonLight,
    AlertButtonDark,
    DisabledButtonLight,
    DisabledButtonDark,
    TabInactive,
    TabActive,
    TabText,
    TabIndicator,
    TextCursorColor,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VMConfigVersion4 {
    pub file_version: String,
    pub menu_shown: Option<bool>,
    pub color_scheme: ColorScheme,
    dark_palette: HashMap<VMColor, (u8,u8,u8,u8)>,
    light_palette: HashMap<VMColor, (u8,u8,u8,u8)>,
}

impl VMConfigVersion4 {
    fn ensure_full_color_schemes(&mut self) {
        let example = VMConfigVersion4::default();
        for (key, color) in &example.light_palette {
            if let None = self.light_palette.get(&key) {
                self.light_palette.insert((*key).clone(), (*color).clone());
            } 
        }
        for (key, color) in &example.dark_palette {
            if let None = self.dark_palette.get(&key) {
                self.dark_palette.insert((*key).clone(), (*color).clone());
            } 
        }
    }
}

pub struct VMConfigSerde;

#[allow(unused_must_use)]
impl VMConfigSerde {
    pub fn load() -> Result<VMConfigVersion4, (String, VMConfigVersion4)> {
        if let Some(mut path) = dirs::config_dir() {
            path.push(DEFAULT_CONFIG_DIR_NAME);
            if !path.clone().exists() {
                if let Ok(_) = fs::create_dir(path.clone()) {
                } else {
                    return Err((
                        format!("Couldn't create configuration directory at {:?}", path.clone()),
                        VMConfigVersion4::default()
                    ));
                }
            }
            path.push(DEFAULT_CONFIG_FILE_NAME);
            if !path.exists() {
                println!("No config file found. Creating at {}", path.display());
                let config = VMConfigVersion4::default();
                fs::write(path, serde_json::to_string_pretty(&config).ok().expect("Failed to serialize default config!")).expect("Failed to write default config to file");
                return Ok(config)
            } else {
                if let Ok(string) = fs::read_to_string(path.clone()) {
                    if let Ok(mut config) = serde_json::from_str::<VMConfigVersion4>(&string) {
                        config.ensure_full_color_schemes();
                        return Ok(config);
                    } else {
                        let mut config_path_renamed = path.clone();
                        config_path_renamed.set_extension("old");
                        fs::rename(path, config_path_renamed);
                        let config = VMConfigVersion4::default(); 
                        VMConfigSerde::save(&config);
                        return Err((
                            String::from("Could not serialized config file as any known version"),
                            config
                        ));
                    }
                }
            }
        }
        Err((
            "General filesystem error".to_string(),
            VMConfigVersion4::default()
        ))
    }

    pub fn save(config: &VMConfigVersion4) -> Result<String, String> {
        let mut path = dirs::config_dir().expect("no user config dir found");
        path.push(DEFAULT_CONFIG_DIR_NAME);
        path.push(DEFAULT_CONFIG_FILE_NAME);
        fs::write(path, serde_json::to_string_pretty(&config).ok().expect("Failed to serialize config")).ok().expect("Failed to save config");
        Ok("".to_string())
    }
}

use VMColor::*;
impl Default for VMConfigVersion4 {

    fn default() -> Self {
        let mut dark_palette: HashMap<VMColor, (u8,u8,u8,u8)> = HashMap::new();
        let mut light_palette: HashMap<VMColor, (u8,u8,u8,u8)> = HashMap::new();
        light_palette.insert(LabelTextColor, (0,0,0,255));
        light_palette.insert(DisabledLabelTextColor, (0,0,0,128));
        light_palette.insert(DisabledNodeBorderColor, (0,0,0,128));
        light_palette.insert(NodeBorderColor, (0,0,0,255));
        light_palette.insert(ActiveNodeBorderColor, (125,125,255,255));
        light_palette.insert(TargetNodeBorderColor, (255,125,125,255));
        light_palette.insert(NodeBackgroundColor, (200,200,200,255));
        light_palette.insert(DisabledNodeBackgroundColor, (200,200,200,128));
        light_palette.insert(EdgeColor, (192,192,192,255));
        light_palette.insert(ComposeIndicatorTextColor, (255,0,0,255));
        light_palette.insert(SheetBackgroundColor, (255,255,255,255));
        light_palette.insert(DialogBackgroundColor, (128,128,128,70));
        light_palette.insert(StackBackgroundColor, (128,128,128,140));
        light_palette.insert(AlertColor, (255,0,0,255));
        light_palette.insert(ButtonLight, (33,33,33,255));
        light_palette.insert(ButtonDark, (0,0,0,255));
        light_palette.insert(AlertButtonLight, (128,11,11,255));
        light_palette.insert(AlertButtonDark, (40,0,0,255));
        light_palette.insert(DisabledButtonLight, (56,56,56,255));
        light_palette.insert(DisabledButtonDark, (40,40,40,255));
        light_palette.insert(TabText, (0,0,0,255));
        light_palette.insert(TabActive, (180,180,180,255));
        light_palette.insert(TabInactive, (200,200,200,192));
        light_palette.insert(TabIndicator, (125,125,255,255));
        light_palette.insert(TextCursorColor, (125,125,255,255));
        dark_palette.insert(LabelTextColor, (255,255,255,255));
        dark_palette.insert(DisabledLabelTextColor, (255,255,255,128));
        dark_palette.insert(NodeBorderColor, (215,215,215,255));
        dark_palette.insert(DisabledNodeBorderColor, (215,215,215,128));
        dark_palette.insert(ActiveNodeBorderColor, (125,125,255,255));
        dark_palette.insert(TargetNodeBorderColor, (255,125,125,255));
        dark_palette.insert(NodeBackgroundColor, (100,100,100,255));
        dark_palette.insert(DisabledNodeBackgroundColor, (100,100,100,128));
        dark_palette.insert(EdgeColor, (132,132,132,255));
        dark_palette.insert(ComposeIndicatorTextColor, (255,0,0,255));
        dark_palette.insert(SheetBackgroundColor, (0,0,0,255));
        dark_palette.insert(DialogBackgroundColor, (128,128,128,70));
        dark_palette.insert(StackBackgroundColor, (128,128,128,140));
        dark_palette.insert(AlertColor, (255,0,0,255));
        dark_palette.insert(ButtonLight, (33,33,33,255));
        dark_palette.insert(ButtonDark, (0,0,0,255));
        dark_palette.insert(AlertButtonLight, (128,11,11,255));
        dark_palette.insert(AlertButtonDark, (40,0,0,255));
        dark_palette.insert(DisabledButtonLight, (56,56,56,255));
        dark_palette.insert(DisabledButtonDark, (40,40,40,255));
        dark_palette.insert(TabText, (255,255,255,255));
        dark_palette.insert(TabActive, (60,60,60,255));
        dark_palette.insert(TabInactive, (100,100,100,192));
        dark_palette.insert(TabIndicator, (125,125,255,255));
        dark_palette.insert(TextCursorColor, (50,50,255,255));

        let mut config = VMConfigVersion4 {
            file_version: String::from(CURRENT_CONFIG_FILE_VERSION.to_string()),
            menu_shown: Some(true),
            color_scheme: ColorScheme::LIGHT,
            light_palette,
            dark_palette,
        };

        let system_mode = dark_light::detect();

        match system_mode {
            dark_light::Mode::Light => {
                config.set_color_scheme(ColorScheme::LIGHT);
            }
            dark_light::Mode::Dark => {
                config.set_color_scheme(ColorScheme::DARK);
            }
        }
        config
    }

}

impl VMConfigVersion4 {
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

    pub fn get_color(&self, key: VMColor) -> Result<Color, String> {
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