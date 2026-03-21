class Shrug < Formula
  desc "A dynamic CLI for Atlassian Cloud"
  homepage "https://github.com/mfassaie/shrug"
  version "VERSION"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/mfassaie/shrug/releases/download/vVERSION/shrug-aarch64-apple-darwin.tar.gz"
      sha256 "SHA256"
    else
      url "https://github.com/mfassaie/shrug/releases/download/vVERSION/shrug-x86_64-apple-darwin.tar.gz"
      sha256 "SHA256"
    end
  end

  on_linux do
    url "https://github.com/mfassaie/shrug/releases/download/vVERSION/shrug-x86_64-unknown-linux-musl.tar.gz"
    sha256 "SHA256"
  end

  def install
    bin.install "shrug"
  end

  test do
    assert_match "shrug", shell_output("#{bin}/shrug --version")
  end
end
