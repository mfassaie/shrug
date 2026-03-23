# VHS demo tape for shrug CLI
# Run: vhs demo.tape
# Produces: assets/demo.gif
#
# Prerequisites:
#   - shrug installed and on PATH (cargo install --path .)
#   - A profile configured with Atlassian credentials
#   - vhs, ttyd, ffmpeg installed

Output assets/demo.gif
Set FontSize 15
Set Width 1100
Set Height 650
Set Theme "Dracula"
Set TypingSpeed 50ms
Set Padding 20

# Version
Type "shrug --version"
Enter
Sleep 2s

# Help overview
Type "shrug --help"
Enter
Sleep 4s

# List Jira projects
Type "shrug --output json jira projects get-all-projects"
Enter
Sleep 4s

# Create a Jira issue
Type "shrug --output json --json '{"
Type@0ms '"fields":{"project":{"key":"TEST"},"summary":"Created from shrug CLI","issuetype":{"name":"Task"}}}'
Type "' jira issues create-issue"
Enter
Sleep 4s

# List Confluence pages
Type "shrug --output json confluence page get-pages"
Enter
Sleep 4s

# Shell completions
Type "shrug completions bash | head -5"
Enter
Sleep 3s

Sleep 2s
