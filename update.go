package main

import (
	"archive/zip"
	"encoding/json"
	"errors"
	"io"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"strconv"
	"strings"
)

var (
	installerCdn = "https://setup.rbxcdn.com/"

	appSettingsXML = `<?xml version="1.0" encoding="UTF-8"?>
<Settings>
	<ContentFolder>content</ContentFolder>
	<BaseUrl>http://www.roblox.com</BaseUrl>
</Settings>	
`
)

type ClientVersion struct {
	Version             string `json:"version"`
	ClientVersionUpload string `json:"clientVersionUpload"`
	BootstrapperVersion string `json:"bootstrapperVersion"`
}

type FileInfo struct {
	name         string
	checksum     string
	zippedSize   uint
	unzippedSize uint
}

func Update() (string, error) {
	clientVersion, err := getClientVersionOnline()
	if err != nil {
		return "", err
	}

	config, err := GetConfig()
	if err != nil {
		return "", err
	}

	currVersionPath := filepath.Join(config.Roblox.InstallationDir, clientVersion.ClientVersionUpload)
	if _, err = os.Stat(currVersionPath); !os.IsNotExist(err) {
		return clientVersion.ClientVersionUpload, nil
	}

	files := getPackageManifest(clientVersion.ClientVersionUpload)
	installDir := filepath.Join(config.Roblox.InstallationDir, clientVersion.ClientVersionUpload)

	for _, file := range files {
		log.Printf("Downloading %v", file.name)
		if err := downloadFile(file, installDir); err != nil {
			return "", err
		}

		ext := filepath.Ext(file.name)
		if ext == ".zip" {
			dest := strings.TrimSuffix(file.name, ext)

			// Some content files need to be extracted in PlatformContent/pc directory
			switch dest {
			case "content-platform-fonts":
				dest = filepath.Join("PlatformContent", "pc", "fonts")

			case "content-terrain":
				dest = filepath.Join("PlatformContent", "pc", "terrain")

			case "content-textures3":
				dest = filepath.Join("PlatformContent", "pc", "textures")

			case "content-textures2":
				dest = filepath.Join("content", "textures")

			case "RobloxApp":
				dest = ""

			default:
				if res := strings.Split(dest, "-"); len(res) > 1 {
					dest = filepath.Join(res...)
				}
			}

			zipname := filepath.Join(installDir, file.name)
			if err = unzipFile(zipname, filepath.Join(installDir, dest)); err != nil {
				return "", err
			}

			os.Remove(zipname)
		}
	}

	// Roblox requires AppSettings.xml to run
	err = os.WriteFile(filepath.Join(installDir, "AppSettings.xml"), []byte(appSettingsXML), 0777)
	if err != nil {
		return "", nil
	}

	return clientVersion.ClientVersionUpload, nil
}

func getClientVersionOnline() (ClientVersion, error) {
	resp, err := http.Get("https://clientsettingscdn.roblox.com/v2/client-version/WindowsPlayer")
	if err != nil {
		log.Fatalln(err)
	}

	if resp.StatusCode != http.StatusOK {
		return ClientVersion{}, errors.New("unable to fetch client version")
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		log.Fatalln(err)
	}

	var clientVersion ClientVersion
	if err = json.Unmarshal(body, &clientVersion); err != nil {
		return clientVersion, err
	}

	return clientVersion, nil
}

func getPackageManifest(version string) []FileInfo {
	installerCdn += version

	resp, err := http.Get(installerCdn + "-rbxPkgManifest.txt")
	if err != nil {
		log.Fatalln(err)
	}

	manifest, err := io.ReadAll(resp.Body)
	if err != nil {
		log.Fatalln(err)
	}

	// Unmarshall package manifest
	lists := strings.Split(string(manifest[:]), "\r\n")
	lists = lists[1 : len(lists)-1]

	fileInfos := make([]FileInfo, 0)
	for i := 0; i < len(lists)-1; i += 4 {
		zippedSize, err := strconv.ParseUint(lists[i+2], 10, 32)
		if err != nil {
			log.Fatalln(err)
		}

		unzippedSize, err := strconv.ParseUint(lists[i+3], 10, 32)
		if err != nil {
			log.Fatalln(err)
		}

		fileInfo := FileInfo{
			lists[i],
			lists[i+1],
			uint(zippedSize),
			uint(unzippedSize),
		}

		fileInfos = append(fileInfos, fileInfo)
	}

	return fileInfos
}

func downloadFile(file FileInfo, dest string) error {
	if err := os.MkdirAll(dest, 0777); err != nil {
		return err
	}

	out, err := os.Create(filepath.Join(dest, file.name))
	if err != nil {
		return err
	}
	defer out.Close()

	resp, err := http.Get(installerCdn + "-" + file.name)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	_, err = io.Copy(out, resp.Body)
	if err != nil {
		return err
	}

	return nil
}

func unzipFile(source, destination string) error {
	reader, err := zip.OpenReader(source)
	if err != nil {
		return err
	}
	defer reader.Close()

	extractFile := func(file *zip.File) error {
		rc, err := file.Open()
		if err != nil {
			return nil
		}
		defer rc.Close()

		path := filepath.Join(destination, file.Name)

		if file.FileInfo().IsDir() {
			if err = os.MkdirAll(path, file.Mode()); err != nil {
				return err
			}

		} else {
			if err = os.MkdirAll(filepath.Dir(path), file.Mode()); err != nil {
				return err
			}

			out, err := os.Create(path)
			if err != nil {
				return err
			}
			defer out.Close()

			if _, err = io.Copy(out, rc); err != nil {
				return err
			}
		}

		return nil
	}

	for _, file := range reader.File {
		if err := extractFile(file); err != nil {
			return err
		}
	}

	return nil
}

// Source: https://gist.github.com/jerblack/d0eb182cc5a1c1d92d92a4c4fcc416c6#file-elevate-go-L20
// func runAsAdmin() {
// 	verb := "runas"
// 	exe, _ := os.Executable()
// 	cwd, _ := os.Getwd()
// 	args := strings.Join(os.Args[1:], " ")

// 	verbPtr, _ := syscall.UTF16PtrFromString(verb)
// 	exePtr, _ := syscall.UTF16PtrFromString(exe)
// 	cwdPtr, _ := syscall.UTF16PtrFromString(cwd)
// 	argPtr, _ := syscall.UTF16PtrFromString(args)

// 	var showCmd int32 = 1 //SW_NORMAL

// 	err := windows.ShellExecute(0, verbPtr, exePtr, argPtr, cwdPtr, showCmd)
// 	if err != nil {
// 		log.Fatalln(err)
// 	}
// }

// func runningAsAdmin() bool {
// 	_, err := os.Open(`\\.\PHYSICALDRIVE0`)
// 	return err == nil
// }
