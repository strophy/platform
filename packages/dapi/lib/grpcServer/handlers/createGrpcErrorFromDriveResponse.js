const cbor = require('cbor');

const {
  server: {
    error: {
      InternalGrpcError,
      InvalidArgumentGrpcError,
      DeadlineExceededGrpcError,
      ResourceExhaustedGrpcError,
      NotFoundGrpcError,
      FailedPreconditionGrpcError,
      UnavailableGrpcError,
      GrpcError,
    },
  },
} = require('@dashevo/grpc-common');
const GrpcErrorCodes = require('@dashevo/grpc-common/lib/server/error/GrpcErrorCodes');
const AlreadyExistsGrpcError = require('@dashevo/grpc-common/lib/server/error/AlreadyExistsGrpcError');
const createConsensusError = require('@dashevo/dpp/lib/errors/consensus/createConsensusError');

/**
 * @param {Object} data
 * @returns {{"drive-error-data-bin": Buffer}||{}}
 */
function createRawMetadata(data) {
  if (Object.keys(data).length === 0) {
    return {};
  }

  return {
    'drive-error-data-bin': cbor.encode(data),
  };
}

const COMMON_ERROR_CLASSES = {
  [GrpcErrorCodes.INVALID_ARGUMENT]: InvalidArgumentGrpcError,
  [GrpcErrorCodes.DEADLINE_EXCEEDED]: DeadlineExceededGrpcError,
  [GrpcErrorCodes.NOT_FOUND]: NotFoundGrpcError,
  [GrpcErrorCodes.ALREADY_EXISTS]: AlreadyExistsGrpcError,
  [GrpcErrorCodes.RESOURCE_EXHAUSTED]: ResourceExhaustedGrpcError,
  [GrpcErrorCodes.FAILED_PRECONDITION]: FailedPreconditionGrpcError,
  [GrpcErrorCodes.UNAVAILABLE]: UnavailableGrpcError,
};

/**
 * @typedef createGrpcErrorFromDriveResponse
 * @param {number} code
 * @param {string} info
 * @return {GrpcError}
 */
function createGrpcErrorFromDriveResponse(code, info) {
  if (code === undefined) {
    return new InternalGrpcError(new Error('Drive’s error code is empty'));
  }

  const decodedInfo = info ? cbor.decode(Buffer.from(info, 'base64')) : { };

  // gRPC error codes
  if (code <= 16) {
    const CommonErrorClass = COMMON_ERROR_CLASSES[code.toString()];
    if (CommonErrorClass) {
      return new CommonErrorClass(
        decodedInfo.message,
        createRawMetadata(decodedInfo.data),
      );
    }

    // Restore stack for internal error
    if (code === GrpcErrorCodes.INTERNAL) {
      const error = new Error(decodedInfo.message);
      error.stack = decodedInfo.data.stack;

      delete decodedInfo.data.stack;

      return new InternalGrpcError(error, createRawMetadata(decodedInfo.data));
    }

    return new GrpcError(
      code,
      decodedInfo.message,
      createRawMetadata(decodedInfo.data),
    );
  }

  // Undefined Drive and DAPI errors
  if (code >= 17 && code < 1000) {
    return new GrpcError(
      GrpcErrorCodes.UNKNOWN,
      decodedInfo.message,
      createRawMetadata(decodedInfo.data),
    );
  }

  // DPP errors
  if (code >= 1000 && code < 5000) {
    const consensusError = createConsensusError(code, decodedInfo.data.arguments || []);

    // Basic
    if (code >= 1000 && code < 2000) {
      return new InvalidArgumentGrpcError(
        consensusError.message,
        { code, ...createRawMetadata(decodedInfo.data) },
      );
    }

    // Signature
    if (code >= 2000 && code < 3000) {
      return new GrpcError(
        GrpcErrorCodes.UNAUTHENTICATED,
        consensusError.message,
        { code, ...createRawMetadata(decodedInfo.data) },
      );
    }

    // Fee
    if (code >= 3000 && code < 4000) {
      return new FailedPreconditionGrpcError(
        consensusError.message,
        { code, ...createRawMetadata(decodedInfo.data) },
      );
    }

    // State
    if (code >= 4000 && code < 5000) {
      return new InvalidArgumentGrpcError(
        consensusError.message,
        { code, ...createRawMetadata(decodedInfo.data) },
      );
    }
  }

  return new InternalGrpcError(new Error(`Unknown Drive’s error code: ${code}`));
}

module.exports = createGrpcErrorFromDriveResponse;
