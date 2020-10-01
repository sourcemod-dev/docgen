import sys
try:
	from ambuild2 import run, util
except:
	try:
		import ambuild
		sys.stderr.write('It looks like you have AMBuild 1 installed, but this project uses AMBuild 2.\n')
		sys.stderr.write('Upgrade to the latest version of AMBuild to continue.\n')
	except:
		sys.stderr.write('AMBuild must be installed to build this project.\n')
		sys.stderr.write('http://www.alliedmods.net/ambuild\n')
	sys.exit(1)

def make_objdir_name(p):
  return 'obj-' + util.Platform() + '-' + p.target_arch

parser = run.BuildParser(sourcePath=sys.path[0], api='2.1')
parser.default_arch = 'x64'
parser.default_build_folder = make_objdir_name
parser.options.add_option('--enable-debug', action='store_const', const='1', dest='debug',
                       help='Enable debugging symbols')
parser.options.add_option('--enable-optimize', action='store_const', const='1', dest='opt',
                       help='Enable optimization')
parser.options.add_option('--amtl', type='string', dest='amtl', default=None, help='Custom AMTL path')
parser.options.add_option('--build', type='string', dest='build', default='all', 
                       help='Build which components (all, spcomp, vm, exp, test, core)')
parser.options.add_option('--enable-spew', action='store_true', default=False, dest='enable_spew',
                       help='Enable debug spew')
parser.options.add_option("--enable-coverage", action='store_true', default=False,
                       dest='enable_coverage', help='Enable code coverage support.')
parser.Configure()