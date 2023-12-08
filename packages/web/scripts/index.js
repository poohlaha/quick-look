/**
 * @fileOverview 项目打包
 * @date 2023-04-12
 * @author poohlaha
 */
const { Paths } = require('@bale-tools/utils')
const MutateVersion = require('@bale-tools/mutate-version')
const chalk = require('chalk')
const { performance } = require('node:perf_hooks')
const path = require('node:path')
const ProjectBuilder = require('./build')

const LoggerPrefix = chalk.cyan('[Bale Chat Compiler]:')

class Builder {
  _SCRIPTS = ['start', 'dev', 'prod'] // scripts
  _appRootDir = ''
  _packageDir = ''
  _configurationsDir = ''
  _copyDir = ''
   _pcDestDir = ''
   _mobileDestDir = ''
  _mutateVersion
  _args = []
  _command = 2 // default dev
  _script // default dev
  _descDir // dest dir
  _projectBuilder // project builder

  constructor() {
    this._appRootDir = Paths.getAppRootDir() || ''
    this._packageDir = path.join(this._appRootDir, 'packages')
    this._configurationsDir = path.join(this._appRootDir, 'configurations')
    this._copyDir = path.resolve(this._appRootDir, 'node_modules', '@bale-sprint/react')
    this._args = process.argv.slice(2) || []
    this._command = this._getCommand()
    this._script = this._getScript()
    this._descDir = this._script === this._SCRIPTS[2] ? 'dist' : 'build'
    this._pcDestDir = path.join(this._packageDir, 'pc')
    this._mobileDestDir = path.join(this._packageDir, 'mobile')
    this._projectBuilder = new ProjectBuilder()
    this._mutateVersion = new MutateVersion({ language: 'react', babelImportPluginName: '', useTypescript: true })
  }


  // 获取输入命令 0: clean 1: copy files 2: start 3: dev 4: prod
  _getCommand() {
    const commands = this._args.filter(x => !x.startsWith('--')) || []
    if (commands.includes('clean')) {
      return 0
    }

    if (commands.includes('copy')) {
      return 1
    }

    if (commands.includes('start')) {
      return 2
    }

    if (commands.includes('dev')) {
      return 3
    }

    if (commands.includes('prod')) {
      return 4
    }

    return 3
  }

  _copyFiles() {
    this._mutateVersion.copy()
  }

  // clean
  _clean() {
    console.log(LoggerPrefix, `Starting ${chalk.cyan('clean')} ...`)
    const startTime = performance.now()

    // clean copyFiles
    this._mutateVersion.clean()

    // clean src files
    this._projectBuilder.copy(true)
    const endTime = performance.now()
    console.log(LoggerPrefix, `Finished ${chalk.cyan('clean')} after ${chalk.magenta(`${endTime - startTime} ms`)}`)
  }

  // get script
  _getScript() {
    const commands = this._args.filter(x => !x.startsWith('--')) || []
    if (commands.includes(`${this._SCRIPTS[0]}`)) {
      return this._SCRIPTS[0]
    }

    if (commands.includes(`${this._SCRIPTS[1]}`)) {
      return this._SCRIPTS[1]
    }

    if (commands.includes(`${this._SCRIPTS[2]}`)) {
      return this._SCRIPTS[2]
    }

    return this._SCRIPTS[1]
  }

  // build
  _build() {
    // this._copyFiles()
    this._projectBuilder.instance()
  }

  // instance 0: clean 1: copy files 2: start mobile 3: start pc 4: dev 5: prod
  instance() {
    switch (this._command) {
      case 0: // clean
        this._clean()
        break
      case 1: // copy
        this._copyFiles()
        break
      case 2: // start
      case 3: // dev
      case 4: // prod
      default:
        this._build()
        break
    }
  }
}
module.exports = new Builder().instance()
