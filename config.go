package main

import (
	"errors"
	"io"
	"log"
	"os"
	"os/user"

	"github.com/pelletier/go-toml"
)

type Config struct {
	Roblox       RbxConfig      `toml:"roblox"`
	RichPresence PresenceConfig `toml:"rich_presence"`
}

type RbxConfig struct {
	InstallationDir string `toml:"installation_dir"`
	RbxSecurity     string `toml:"rbx_security"`
}

type PresenceConfig struct {
	DisplayUsername bool `toml:"display_username"`
	DisplayGame     bool `toml:"display_game"`
	EnablePresence  bool `toml:"enable_presence"`
}

func GetConfig() (Config, error) {
	file, err := os.Open("config.toml")
	if os.IsNotExist(err) {
		rbxDir, err := findRbxDir()
		if err != nil {
			log.Fatalln(err)
		}

		config := Config{
			RbxConfig{rbxDir, ""},
			PresenceConfig{true, true, true},
		}

		err = SetConfig(config)
		if err != nil {
			log.Fatalln(err)
		}

		return config, nil

	} else if err != nil {
		log.Fatalln(err)
	}

	defer file.Close()

	content, err := io.ReadAll(file)
	if err != nil {
		return Config{}, err
	}

	var config Config
	err = toml.Unmarshal(content, &config)
	if err != nil {
		return Config{}, err
	}

	return config, nil
}

// TODO: Set config
func SetConfig(config Config) error {
	data, err := toml.Marshal(config)
	if err != nil {
		return err
	}

	err = os.WriteFile("config.toml", data, 0777)
	if err != nil {
		return err
	}

	return nil
}

func findRbxDir() (string, error) {
	currUser, err := user.Current()
	if err != nil {
		log.Fatalln(err)
	}

	possibleDirs := []string{
		`C:\Users\` + currUser.Name + `\AppData\Local\Roblox\Versions`,
		`C:\Program Files\Roblox\Versions`,
		`C:\Program Files (x86)\Roblox\Versions`,
	}

	for _, dir := range possibleDirs {
		if _, err := os.Stat(dir); !os.IsNotExist(err) {
			return dir, nil
		}
	}

	return "", errors.New("unable to find roblox directory")

}
