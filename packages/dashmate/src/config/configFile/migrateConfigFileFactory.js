const semver = require('semver');

function migrateConfigFileFactory(getConfigFileMigrations) {
  /**
   * @typedef {function} migrateConfigFile
   * @param {Object} rawConfigFile
   * @param {string} fromVersion
   * @param {string} toVersion
   * @returns {Object}
   */
  function migrateConfigFile(rawConfigFile, fromVersion, toVersion) {
    if (fromVersion === toVersion) {
      return rawConfigFile;
    }

    const configFileMigrations = getConfigFileMigrations();

    return Object.keys(configFileMigrations)
      .filter((version) => (semver.gt(version, fromVersion) && semver.lte(version, toVersion)))
      .sort(semver.compare)
      .reduce((migratedOptions, version) => {
        const migrationFunction = configFileMigrations[version];
        return migrationFunction(rawConfigFile);
      }, rawConfigFile);
  }

  return migrateConfigFile;
}

module.exports = migrateConfigFileFactory;
