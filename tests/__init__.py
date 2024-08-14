"""
Unit tests for the project.

This module sets up and runs the test suite for the project.
It can be run locally or as part of a GitHub Actions workflow.
"""
import unittest
import os
import sys

# Add the parent directory to sys.path to allow importing from the project
project_root = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
sys.path.insert(0, project_root)

def run_tests():
    """
    Discover and run all tests in the 'tests' directory.
    """
    loader = unittest.TestLoader()
    suite = loader.discover('tests')
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)
    return result

if __name__ == '__main__':
    result = run_tests()
    sys.exit(not result.wasSuccessful())

# GitHub Actions workflow
def github_actions_run():
    """
    Run tests and set the exit code for GitHub Actions.
    """
    result = run_tests()
    if not result.wasSuccessful():
        print("::set-output name=test_outcome::failure")
        sys.exit(1)
    else:
        print("::set-output name=test_outcome::success")
        sys.exit(0)

if os.environ.get('GITHUB_ACTIONS') == 'true':
    github_actions_run()
