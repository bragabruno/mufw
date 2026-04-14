class Mufw < Formula
  desc "ufw-like command-line firewall for macOS (pf)"
  homepage "https://github.com/bragdev/mufw"
  url "https://github.com/bragdev/mufw/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_ON_RELEASE"
  license any_of: ["MIT", "Apache-2.0"]
  head "https://github.com/bragdev/mufw.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/mufw-cli")
  end

  service do
    run [opt_bin/"mufw", "enable"]
    keep_alive false
    run_at_load true
    log_path var/"log/mufw.log"
    error_log_path var/"log/mufw.log"
  end

  test do
    assert_match "mufw", shell_output("#{bin}/mufw --version")
  end
end
