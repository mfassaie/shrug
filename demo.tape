# shrug — dynamic CLI for Atlassian Cloud
# https://github.com/mfassaie/shrug

Output assets/demo.gif

Require shrug

Set Shell "bash"
Set FontSize 14
Set Width 1000
Set Height 600
Set Padding 20
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set WindowBarSize 40
Set TypingSpeed 40ms
Set CursorBlink false

# Create a Jira issue
Type "shrug jira +create --project KAN --summary 'Fix login timeout' --type Bug"
Enter
Sleep 3s

# Search for issues in the project
Type "shrug jira +search --project KAN"
Enter
Sleep 3s

# Create a Confluence page
Type@50ms "shrug confluence +create --space TESTSPACE --title 'Release notes' --body '## v1.0'"
Enter
Sleep 3s

# List Confluence pages
Type "shrug confluence page get-pages --limit 3"
Enter
Sleep 4s

# Show what shrug supports
Type "shrug --help"
Enter
Sleep 3s
