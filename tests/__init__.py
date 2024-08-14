"""
Unit test
"""
import sys
import unittest

sys.path.append("../thongna")

loader = unittest.TestLoader()
testSuite = loader.discover("tests")
testRunner = unittest.TextTestRunner(verbosity=1)
testRunner.run(testSuite)
