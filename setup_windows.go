//go:build windows

package main

import (
	"fmt"
	"os"

	"golang.org/x/sys/windows/registry"
)

// Setup registry to use `roblox-player` protocol
func setup() error {
	execPath, err := os.Executable()
	if err != nil {
		return err
	}

	// Create `SOFTWARE\Classes\roblox-player` key and register it as a URL Protocol
	newK, _, err := registry.CreateKey(registry.CURRENT_USER, `roblox-player`, registry.WRITE)
	if err != nil {
		return fmt.Errorf("could not create registry key roblox-player: %v", err)
	}
	defer newK.Close()

	err = newK.SetStringValue("", "URL: Roblox Protocol with Rich Presence")
	if err != nil {
		return fmt.Errorf("could not set default value of key roblox-player: %v", err)
	}

	err = newK.SetStringValue("URL Protocol", "")
	if err != nil {
		return fmt.Errorf("could not set value 'URL Protocol' of key roblox-player: %v", err)
	}

	// Create `SOFTWARE\Classes\roblox-player\DefaultIcon` key
	newK, _, err = registry.CreateKey(registry.CURRENT_USER, `roblox-player\DefaultIcon`, registry.WRITE)
	if err != nil {
		return fmt.Errorf(`could not create registry key roblox-player\DefaultIcon: %v`, err)
	}
	defer newK.Close()

	err = newK.SetStringValue("", execPath)
	if err != nil {
		return fmt.Errorf(`could not set registry key roblox-player\DefaultIcon: %v`, err)
	}

	// Create `SOFTWARE\Classes\roblox-player\shell\open\command` key
	newK, _, err = registry.CreateKey(registry.CURRENT_USER, `roblox-player\shell\open\command`, registry.WRITE)
	if err != nil {
		return fmt.Errorf(`could not create registry key roblox-player\shell\open\command: %v`, err)
	}
	defer newK.Close()

	err = newK.SetStringValue("", fmt.Sprintf(`"%s" "%%1"`, execPath))
	if err != nil {
		return fmt.Errorf(`could not set default value of key roblox-player\shell\open\command: %v`, err)
	}

	return nil
}
