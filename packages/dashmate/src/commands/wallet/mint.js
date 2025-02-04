const { Listr } = require('listr2');

const { Flags } = require('@oclif/core');

const ConfigBaseCommand = require('../../oclif/command/ConfigBaseCommand');
const MuteOneLineError = require('../../oclif/errors/MuteOneLineError');

const { NETWORK_LOCAL } = require('../../constants');

class MintCommand extends ConfigBaseCommand {
  /**
   * @param {Object} args
   * @param {Object} flags
   * @param {generateToAddressTask} generateToAddressTask
   * @param {Config} config
   * @return {Promise<void>}
   */
  async runWithDependencies(
    {
      amount,
    },
    {
      address,
      verbose: isVerbose,
    },
    generateToAddressTask,
    config,
  ) {
    const network = config.get('network');

    if (network !== NETWORK_LOCAL) {
      throw new Error('Only local network supports generation of dash');
    }

    const tasks = new Listr([
      {
        title: `Generate ${amount} dash to address`,
        task: () => generateToAddressTask(config, amount),
      },
    ],
    {
      renderer: isVerbose ? 'verbose' : 'default',
      rendererOptions: {
        showTimer: isVerbose,
        clearOutput: false,
        collapse: false,
        showSubtasks: true,
      },
    });

    try {
      await tasks.run({
        address,
        network,
      });
    } catch (e) {
      throw new MuteOneLineError(e);
    }
  }
}

MintCommand.description = `Mint tDash

Mint given amount of tDash to a new or specified address
`;

MintCommand.flags = {
  ...ConfigBaseCommand.flags,
  address: Flags.string({ char: 'a', description: 'use recipient address instead of creating new', default: null }),
};

MintCommand.args = [{
  name: 'amount',
  required: true,
  description: 'amount of tDash to be generated to address',
  parse: (input) => parseInt(input, 10),
}];

module.exports = MintCommand;
