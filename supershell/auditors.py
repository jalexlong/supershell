import os
import stat
import pwd
import subprocess

# --- File System Audit Functions ---

def check_permissions(filepath: str, expected_octal: str) -> bool:
    """Checks the octal permissions of a file. (e.g., '600')"""
    try:
        # Get the full mode (permissions + file type)
        mode = os.stat(filepath).st_mode
        # Isolate the permission bits (last 9 bits)
        permissions = stat.S_IMODE(mode)
        # Convert permissions to a 4-digit octal string (e.g., '0755')
        octal_perm = oct(permissions)[-4:]
        # Compare only the last 3 digits (user, group, other)
        return octal_perm[-3:] == expected_octal
    except FileNotFoundError:
        return False
    except Exception as e:
        print(f"Error checking permissions for {filepath}: {e}")
        return False

def check_owner_user(filepath: str, expected_owner_name: str) -> bool:
    """Checks the owner of a file by username."""
    try:
        stat_info = os.stat(filepath)
        # Uses the 'pwd' module to look up the username from the UID
        owner_name = pwd.getpwuid(stat_info.st_uid).pw_name
        return owner_name == expected_owner_name
    except FileNotFoundError:
        return False
    except KeyError:
        # Handles case where the UID exists but the name lookup fails (shouldn't happen often)
        return False

# --- Network/System Audit Functions (Example Placeholder) ---

def check_network_route(target_ip: str, required_interface: str) -> bool:
    """
    Checks if a network route exists and is configured correctly.
    This is where we'd audit the result of 'ip route show'.
    """
    try:
        # Execute the route command
        result = subprocess.run(
            ['ip', 'route', 'show'],
            capture_output=True, text=True, check=True
        )
        # A placeholder for complex regex checking of routing table
        # Example check: is target_ip reachable via required_interface?
        if target_ip in result.stdout and f"dev {required_interface}" in result.stdout:
            return True
        return False
    except subprocess.CalledProcessError:
        return False # Command failed

# MAPPING of audit function names to actual functions for dynamic loading
AUDIT_FUNCTIONS = {
    "check_permissions": check_permissions,
    "check_owner_user": check_owner_user,
    "check_network_route": check_network_route,
}

