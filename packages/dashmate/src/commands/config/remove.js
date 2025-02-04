const BaseCommand = require('../../oclif/command/BaseCommand');

class ConfigRemoveCommand extends BaseCommand {
  /**
   * @param {Object} args
   * @param {Object} flags
   * @param {ConfigFile} configFile
   * @param {DefaultConfigs} defaultConfigs
   * @return {Promise<void>}
   */
  async runWithDependencies(
    {
      config: configName,
    },
    flags,
    configFile,
    defaultConfigs,
  ) {
    if (defaultConfigs.has(configName)) {
      throw new Error(`system config ${configName} can't be removed.\nPlease use 'dashmate reset --hard --config=${configName}' command to reset the configuration`);
    }

    configFile.removeConfig(configName);

    // eslint-disable-next-line no-console
    console.log(`${configName} removed`);
  }
}

ConfigRemoveCommand.description = 'Remove config';

ConfigRemoveCommand.args = [{
  name: 'config',
  required: true,
  description: 'config name',
}];

module.exports = ConfigRemoveCommand;
