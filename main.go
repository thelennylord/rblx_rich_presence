package main

import (
	"log"
	"net/url"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

type JoinData struct {
	launchMode       string
	gameInfo         string
	placeLauncherUrl string
	robloxLocale     string
	gameLocale       string
}

func main() {
	// Setup logger
	log.SetFlags(log.LstdFlags | log.Lmicroseconds | log.Lshortfile)
	file, err := os.Create("log.txt")
	if err != nil {
		log.Fatalln(err)
	}
	defer file.Close()

	log.SetOutput(file)

	version, err := Update()
	if err != nil {
		// TODO: Should we continue to launch roblox or?
		log.Fatalln(err)
	}

	config, err := GetConfig()
	if err != nil {
		log.Fatalln(err)
	}

	// Handle join arguments
	joinUrl := os.Args[1]
	log.Printf("Using join url: %s", joinUrl)
	joinData, err := unmarshallJoinUrl(joinUrl)
	if err != nil {
		log.Fatalln(err)
	}
	log.Println(joinData)

	// Check whether security token needs refreshing or not
	if err := RefreshSecurityCookie(&joinData); err != nil {
		log.Fatalln(err)
	}

	rbxPlayer := filepath.Join(config.Roblox.InstallationDir, version, "RobloxPlayerBeta.exe")
	cmd := exec.Command(rbxPlayer,
		"--app",
		"-t", joinData.gameInfo,
		"-j", joinData.placeLauncherUrl,
		"--rloc", joinData.robloxLocale,
		"--gloc", joinData.gameLocale,
	)
	cmd.Start()

	go StartDiscordRpc()
	cmd.Process.Wait()
}

func unmarshallJoinUrl(joinUrl string) (JoinData, error) {
	joinData := JoinData{}

	for _, option := range strings.Split(joinUrl, "+") {
		pair := strings.Split(option, ":")
		if len(pair) < 2 {
			continue
		}

		name, value := pair[0], pair[1]

		switch name {
		case "launchmode":
			joinData.launchMode = value

		case "gameinfo":
			joinData.gameInfo = value

		case "placelauncherurl":
			escapedValue, err := url.QueryUnescape(value)
			if err != nil {
				return joinData, err
			}

			joinData.placeLauncherUrl = escapedValue
		case "robloxLocale":
			joinData.robloxLocale = value

		case "gameLocale":
			joinData.gameLocale = value

		default:
			continue
		}
	}

	return joinData, nil
}
