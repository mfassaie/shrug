# shrug — static CLI for Atlassian Cloud
# https://github.com/mfassaie/shrug
#
# Record with: vhs assets/demo.tape
#
# Requires a configured shrug profile pointing to an Atlassian Cloud site
# with test data in the TEST project and TEST Confluence space.

Output assets/demo.gif

Set Shell "bash"
Set FontSize 14
Set Width 1100
Set Height 650
Set Padding 20
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set WindowBarSize 40
Set TypingSpeed 35ms
Set CursorBlink false

# ── Jira: Create a bug ──
Type "shrug jira issue create -s 'Login page times out after 30s' --project TEST --type Bug --priority High"
Enter
Sleep 3s
Ctrl+L

# ── Jira: View the issue ──
Type "shrug --output json jira issue view TEST-132"
Enter
Sleep 3s
Ctrl+L

# ── Jira: Edit the issue ──
Type "shrug jira issue edit TEST-132 --add-label urgent --assignee @me"
Enter
Sleep 3s
Ctrl+L

# ── Jira: Add a comment ──
Type "shrug jira issue comment create TEST-132 --body 'Reproduced on Chrome and Firefox. Timeout at exactly 30s.'"
Enter
Sleep 3s
Ctrl+L

# ── Jira: List issues with JQL shorthand ──
Type "shrug --output json jira issue list --project TEST --limit 3"
Enter
Sleep 3s
Ctrl+L

# ── Jira: Search with JQL ──
Type "shrug --output json jira search list --project TEST --type Bug --status 'To Do'"
Enter
Sleep 3s
Ctrl+L

# ── Confluence: Create a page with markdown ──
Type@45ms "shrug --output json confluence page create --space-id 12910596 --title 'Sprint 12 Retro' --body '## What went well'"
Enter
Sleep 3s
Ctrl+L

# ── Confluence: View a page ──
Type "shrug --output json confluence page view 14516225"
Enter
Sleep 3s
Ctrl+L

# ── Confluence: Comment on a page ──
Type@45ms "shrug confluence page comment create 14516225 --body 'Lets action the CI pipeline item this sprint'"
Enter
Sleep 3s
Ctrl+L

# ── Confluence: List pages in a space ──
Type "shrug --output json confluence page list --space-id 12910596 --limit 5"
Enter
Sleep 3s
Ctrl+L

# ── Confluence: Create a blog post ──
Type@45ms "shrug --output json confluence blogpost create --space-id 12910596 --title 'v1.0 Released' --body '## Highlights'"
Enter
Sleep 3s
Ctrl+L

# ── Confluence: Search ──
Type "shrug --output json confluence search list --cql 'space = TEST' --limit 3"
Enter
Sleep 3s

Sleep 1s
