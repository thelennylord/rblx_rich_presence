package main

import (
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	drpc "github.com/thelennylord/go-discordrpc"
)

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
		//REVIEW: Should we continue to launch roblox or?
		log.Fatalln(err)
	}

	config, err := GetConfig()
	if err != nil {
		log.Fatalln(err)
	}

	client, err := drpc.New(ClientId)
	if err != nil {
		log.Fatalln(err)
	}
	defer client.Socket.Close()

	// If no join arguments are applied, probably launched from Discord
	var joinData interface{}

	if len(os.Args) < 2 {
		ch := make(chan drpc.ActivityEventData)
		err := client.RegisterEvent(ch, drpc.ActivityJoinEvent)
		if err != nil {
			log.Fatalln(err)
		}

		tried := false

	loop:
		for {
			select {
			case data := <-ch:
				pair := strings.Split(data.Secret, ";")
				placeId, gameId := pair[0], pair[1]
				joinData = DeeplinkJoinData{placeId, gameId}
				break loop

			default:
				if tried {
					os.Exit(0)
				}

				tried = true
				// Try again after 3 seconds
				time.Sleep(3 * time.Second)
			}

		}

	} else {
		joinUrl := os.Args[1]
		log.Printf("Using join url: %s", joinUrl)

		// TODO: Support deep links
		joinData, err = unmarshallJoinUrl(joinUrl)
		if err != nil {
			log.Fatalln(err)
		}
		log.Println(joinData)

	}

	rbxPlayer := filepath.Join(config.Roblox.InstallationDir, version, "RobloxPlayerBeta.exe")
	var cmd *exec.Cmd

	switch t := joinData.(type) {
	case DirectJoinData:
		// Refresh the security cookie
		if err := RefreshSecurityCookie(&t); err != nil {
			log.Fatal(err)
		}

		cmd = exec.Command(rbxPlayer,
			"--app",
			"-t", t.gameInfo,
			"-j", t.placeLauncherUrl,
			"--rloc", t.robloxLocale,
			"--gloc", t.gameLocale,
			"-b", t.browserTrackerId,
			"-channel",
			"znext",
		)

	case DeeplinkJoinData:
		if _, err := GetAuthenticatedUser(); err != nil {
			// TODO: Display the error on the screen
			log.Fatalln("user not authenticated to roblox")
		}

		cmd = exec.Command(rbxPlayer,
			"--app",
			"--deeplink",
			fmt.Sprintf("roblox://experiences/start?placeId=%s&gameInstanceId=%s/", t.placeId, t.gameId),
			"-channel",
			"znext",
		)
	}

	if err := cmd.Start(); err != nil {
		log.Fatal(err)
	}

	go setPresence(client)

	processState, _ := cmd.Process.Wait()
	log.Printf("Roblox exited with code %d", processState.ExitCode())
}
