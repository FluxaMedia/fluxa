cask "fluxa" do
  version "0.1.30"
  sha256 "8e6657cc03dc821f3d7d0d2f86a2656858629b32c100990960b414300498c722"

  url "https://github.com/FluxaMedia/fluxa/releases/download/v#{version}/Fluxa.Desktop_#{version}_universal.dmg"
  name "Fluxa Desktop"
  desc "Cross-platform desktop app for streaming and managing your media library"
  homepage "https://github.com/FluxaMedia/fluxa"

  app "Fluxa Desktop.app"
end
