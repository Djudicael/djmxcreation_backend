import re

content = open("Cargo.toml").read()

new_members = """
resolver = "3"
default-members = [
    "crates/app-error",
    "crates/djmxcreation-backend-axum",
    "crates/repository",
    "crates/app-service",
    "crates/app_core",
    "crates/app_config",
]
"""
content = re.sub(r'\]\n\n\[workspace.dependencies\]', ']\n' + new_members + '\n[workspace.dependencies]', content)

patches = """
[patch.crates-io]
tokio = { git = "https://github.com/wasix-org/tokio.git", branch = "wasix-1.47.0" }
socket2 = { git = "https://github.com/wasix-org/socket2.git", branch = "v0.5.5" }
hyper = { git = "https://github.com/wasix-org/hyper.git", branch = "wasix-1.6.0" }
"""
content += '\n' + patches

open("Cargo.toml", "w").write(content)
