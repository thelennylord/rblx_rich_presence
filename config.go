package main

import (
	"errors"
	"fmt"
	"io"
	"io/fs"
	"log"
	"os"
	"path/filepath"

	"github.com/pelletier/go-toml"
)

type Config struct {
	Roblox       RbxConfig      `toml:"roblox"`
	RichPresence PresenceConfig `toml:"rich_presence"`
}

type RbxConfig struct {
	InstallationDir string `toml:"installation_dir"`
}

type PresenceConfig struct {
	DisplayUsername bool `toml:"display_username"`
	DisplayGame     bool `toml:"display_game"`
	EnablePresence  bool `toml:"enable_presence"`
}

func GetConfig() (Config, error) {
	dir, err := execDir()
	if err != nil {
		return Config{}, err
	}

	file, err := os.Open(filepath.Join(dir, "config.toml"))

	if errors.Is(err, fs.ErrNotExist) {
		rbxDir, err := findRbxDir()
		if err != nil {
			log.Fatalln(err)
		}

		config := Config{
			RbxConfig{rbxDir},
			PresenceConfig{true, true, true},
		}

		err = config.Save()
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
		return Config{}, fmt.Errorf("failed to read config.toml: %v", err)
	}

	var config Config
	err = toml.Unmarshal(content, &config)
	if err != nil {
		return Config{}, fmt.Errorf("failed to marshal config.toml (likely malformed toml?): %v", err)
	}

	return config, nil
}

// TODO: Set config
func (config Config) Save() error {
	data, err := toml.Marshal(config)
	if err != nil {
		return err
	}

	dir, err := execDir()
	if err != nil {
		return err
	}

	err = os.WriteFile(filepath.Join(dir, "config.toml"), data, 0644)
	if err != nil {
		return fmt.Errorf("could not save config: %v", err)
	}

	return nil
}

func findRbxDir() (string, error) {
	userCacheDir, err := os.UserCacheDir()
	if err != nil {
		log.Fatalf("could not get UserCacheDir: %v", err)
	}

	possibleDirs := []string{
		filepath.Join(userCacheDir, "Roblox", "Versions"),
		`C:\Program Files\Roblox\Versions`,
		`C:\Program Files (x86)\Roblox\Versions`,
	}

	for _, dir := range possibleDirs {
		log.Printf("searching for Roblox in %s", dir)

		if _, err = os.Stat(dir); err == nil {
			log.Printf("found Roblox in %s", dir)
			return dir, nil

		} else if !errors.Is(err, fs.ErrNotExist) {
			return "", fmt.Errorf("error occurred while searching for Roblox in %s: %v", dir, err)
		}

	}

	return "", errors.New("unable to find Roblox directory")

}
