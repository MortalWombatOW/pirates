import os
import sys
import shutil
import yaml     # For parsing Frontmatter (requires PyYAML)
import glob

# Try to import frontmatter, handle if missing
try:
    import frontmatter
except ImportError:
    print("Error: 'python-frontmatter' library is required. Please install it via pip.")
    sys.exit(1)

ROOT_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../"))
AI_DIR = os.path.join(ROOT_DIR, ".ai")
WORKFLOWS_DIR = os.path.join(AI_DIR, "workflows")
RULES_DIR = os.path.join(AI_DIR, "rules")

# Configuration Paths
ANTIGRAVITY_CONFIG = os.path.join(AI_DIR, "config/antigravity")
CLAUDE_CONFIG = os.path.join(AI_DIR, "config/claude")
GEMINI_CONFIG = os.path.join(AI_DIR, "config/gemini")

# Target Paths in Root
TARGET_AGENT = os.path.join(ROOT_DIR, ".agent")
TARGET_CLAUDE = os.path.join(ROOT_DIR, ".claude")
TARGET_GEMINI = os.path.join(ROOT_DIR, ".gemini")

def create_symlink(source, link_name):
    """Creates a symlink, removing existing files/links if necessary."""
    if os.path.exists(link_name) or os.path.islink(link_name):
        if os.path.islink(link_name):
            os.unlink(link_name)
        elif os.path.isdir(link_name):
            shutil.rmtree(link_name)
        else:
            os.remove(link_name)
    
    # Use relative path for the symlink target to ensure portability
    relative_source = os.path.relpath(source, os.path.dirname(link_name))
    os.symlink(relative_source, link_name)
    print(f"Linked {link_name} -> {relative_source}")

def setup_internal_symlinks():
    """Sets up symlinks within the .ai directory structure."""
    print("Setting up internal symlinks...")
    
    # Antigravity
    create_symlink(RULES_DIR, os.path.join(ANTIGRAVITY_CONFIG, "rules"))
    create_symlink(WORKFLOWS_DIR, os.path.join(ANTIGRAVITY_CONFIG, "workflows"))
    
    # Claude
    create_symlink(WORKFLOWS_DIR, os.path.join(CLAUDE_CONFIG, "commands"))

def generate_gemini_config():
    """Generates Gemini TOML configuration from Markdown workflows."""
    print("Generating Gemini configuration...")
    
    commands_dir = os.path.join(GEMINI_CONFIG, "commands")
    if os.path.exists(commands_dir):
        shutil.rmtree(commands_dir)
    os.makedirs(commands_dir)

    # 1. Generate TOML commands from Workflows
    for md_file in glob.glob(os.path.join(WORKFLOWS_DIR, "*.md")):
        filename = os.path.basename(md_file)
        name, _ = os.path.splitext(filename)
        
        with open(md_file, "r", encoding="utf-8") as f:
            post = frontmatter.load(f)
            
        description = post.metadata.get("description", f"Run the {name} workflow")
        prompt = post.content
        
        # Escape triple quotes in the prompt to avoid breaking the TOML string
        safe_prompt = prompt.replace('"""', '\"""')
        
        toml_content = f'description = "{description}"\n\nprompt = """\n{safe_prompt}\n"""\n'
        
        toml_path = os.path.join(commands_dir, f"{name}.toml")
        with open(toml_path, "w", encoding="utf-8") as f:
            f.write(toml_content)
        print(f"Generated {toml_path}")

    # 2. Generate GEMINI.md (Main Config)
    # It should instruct the agent to read the shared rules.
    gemini_md_content = """# Gemini Agent Configuration

## Shared Rules
The following rules are strictly enforced. The agent usually reads these from `.ai/rules/`.
"""
    # Embed rules directly or reference them? 
    # Gemini CLI usually reads the current directory or a specific file. 
    # The prompt context usually includes the GEMINI.md.
    # To ensure it follows rules, we can explicitly list the rule files to read.
    
    gemini_md_content += "\nYou must respect the rules defined in:\n"
    for rule_file in glob.glob(os.path.join(RULES_DIR, "*.md")):
        relative_rule = os.path.relpath(rule_file, GEMINI_CONFIG)
        gemini_md_content += f"- `.gemini/{relative_rule}`\n"

    with open(os.path.join(GEMINI_CONFIG, "GEMINI.md"), "w", encoding="utf-8") as f:
        f.write(gemini_md_content)
    print("Generated GEMINI.md")
    
    # Symlink rules into .gemini/rules so the relative paths in GEMINI.md work
    create_symlink(RULES_DIR, os.path.join(GEMINI_CONFIG, "rules"))


def setup_root_symlinks():
    """Sets up the top-level symlinks for the tools."""
    print("Setting up root symlinks...")
    create_symlink(ANTIGRAVITY_CONFIG, TARGET_AGENT)
    create_symlink(CLAUDE_CONFIG, TARGET_CLAUDE)
    create_symlink(GEMINI_CONFIG, TARGET_GEMINI)

def main():
    setup_internal_symlinks()
    generate_gemini_config()
    setup_root_symlinks()
    print("Deployment complete.")

if __name__ == "__main__":
    main()
