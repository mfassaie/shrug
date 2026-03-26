class Shrug < Formula
  desc "A static CLI for Atlassian Cloud"
  homepage "https://github.com/mfassaie/shrug"
  version "VERSION"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/mfassaie/shrug/releases/download/vVERSION/shrug-aarch64-apple-darwin.tar.gz"
      sha256 "SHA256_AARCH64_APPLE_DARWIN"
    else
      url "https://github.com/mfassaie/shrug/releases/download/vVERSION/shrug-x86_64-apple-darwin.tar.gz"
      sha256 "SHA256_X86_64_APPLE_DARWIN"
    end
  end

  on_linux do
    url "https://github.com/mfassaie/shrug/releases/download/vVERSION/shrug-x86_64-unknown-linux-musl.tar.gz"
    sha256 "SHA256_X86_64_LINUX_MUSL"
  end

  def install
    bin.install "shrug"
  end

  test do
    assert_match "shrug", shell_output("#{bin}/shrug --version")
  end
end
