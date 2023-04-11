package main

import (
	"errors"
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
		log.Fatalf("Error while updating Roblox: %v", err)
	}

	config, err := GetConfig()
	if err != nil {
		log.Fatalln(err)
	}

	// Connect to Discord IPC
	client, err := drpc.New(ClientId)
	if err != nil {
		log.Fatalf("Couldn't connect to Discord IPC: %v", err)
	}
	defer client.Socket.Close()

	// If no join arguments are applied, probably launched from Discord
	var joinData interface{}

	// No command line argument has been passed, so begin setting up necessary stuff
	if len(os.Args) < 2 {
		// Register the game to Discord
		log.Println("No arguments provided; beginning to set up...")

		executablePath, err := os.Executable()
		if err != nil {
			log.Fatalln(err)
		}

		err = client.RegisterCommand(fmt.Sprintf(`"%s" -d`, executablePath), executablePath)
		if err != nil {
			log.Fatalln(err)
		}

		log.Println("Registered command to Discord")

		// TODO: Replace Roblox URL protocol's launch path to rblx_rich_presence
		log.Println("Setup done; exiting...")
		os.Exit(0)
	}

	switch os.Args[1] {

	case "--discord":
		// User is launching from Discord, so subscribe to the ACTIVITY_JOIN event and fetch the party info
		log.Println("Detected launch from Discord")

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
				log.Printf("Joining using the party secret: %s", data.Secret)

				pair := strings.Split(data.Secret, ";")
				placeId, gameId := pair[0], pair[1]
				joinData = DeeplinkJoinData{placeId, gameId}
				break loop

			default:
				if tried {
					// Discord did not send any party info, so exit with failure
					log.Println("Did not receive party information from Discord")
					os.Exit(1)
				}

				tried = true
				// Try again after 3 seconds
				log.Println("Couldn't get any party information, trying again in 3 seconds...")
				time.Sleep(3 * time.Second)
			}
		}

	default:
		// Assume the argument is a Roblox join protocol
		// TODO: Add checks to confirm
		log.Println("Detected launch from Roblox")

		joinUrl := os.Args[1]
		log.Printf("Using join url: %s", joinUrl)

		// TODO: Support deep links
		joinData, err = unmarshallJoinUrl(joinUrl)
		if err != nil {
			log.Fatalf("Error while unmarshalling join url: %v", err)
		}

		log.Printf("Launching Roblox with join data: %v", joinData)
	}

	rbxPlayer := filepath.Join(config.Roblox.InstallationDir, version, "RobloxPlayerBeta.exe")
	var cmd *exec.Cmd

	switch t := joinData.(type) {
	case DirectJoinData:
		// Refresh the security cookie
		if err := RefreshSecurityCookie(&t); err != nil {
			if !errors.Is(err, ErrTicketRedemption) {
				log.Fatal(err)
			}
		}

		t.gameInfo, err = GetAuthenticationTicket()
		if err != nil {
			log.Fatalf("Couldn't get authentication ticket: %v", err)
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
			log.Fatalln("User is not authenticated to Roblox")
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
		log.Fatalf("Roblox exited with error: %v", err)
	}

	go setPresence(client)

	processState, _ := cmd.Process.Wait()
	log.Printf("Roblox exited with code %d", processState.ExitCode())
}
