const chalk = require('chalk');

const { Flags } = require('@oclif/core');
const { OUTPUT_FORMATS } = require('../../constants');

const printArrayOfObjects = require('../../printers/printArrayOfObjects');

const ConfigBaseCommand = require('../../oclif/command/ConfigBaseCommand');

class ServicesStatusCommand extends ConfigBaseCommand {
  /**
   * @param {Object} args
   * @param {Object} flags
   * @param {DockerCompose} dockerCompose
   * @param {Config} config
   * @param {getServicesScope} getServicesScope
   * @return {Promise<void>}
   */
  async runWithDependencies(
    args,
    flags,
    dockerCompose,
    config,
    getServicesScope,
  ) {
    const scope = await getServicesScope(config);

    const outputRows = [];

    for (const [serviceName, service] of Object.entries(scope)) {
      const {
        title, containerId, image, status,
      } = service;
      if (flags.format === OUTPUT_FORMATS.PLAIN) {
        let statusText;
        if (status) {
          statusText = (status === 'running' ? chalk.green : chalk.red)(status);
        }

        outputRows.push({
          Service: title || 'n/a',
          'Container ID': containerId ? containerId.slice(0, 12) : 'n/a',
          Image: image || 'n/a',
          Status: statusText || 'n/a',
        });
      } else {
        outputRows.push({
          service: serviceName,
          containerId,
          image,
          status,
        });
      }
    }

    printArrayOfObjects(outputRows, flags.format);
  }
}

ServicesStatusCommand.description = 'Show service status details';

ServicesStatusCommand.flags = {
  ...ConfigBaseCommand.flags,
  format: Flags.string({
    description: 'display output format',
    default: OUTPUT_FORMATS.PLAIN,
    options: Object.values(OUTPUT_FORMATS),
  }),
};

module.exports = ServicesStatusCommand;
