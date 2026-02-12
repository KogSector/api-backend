/**
 * API Backend gRPC Clients
 * 
 * Centralized gRPC client management for all downstream services
 */

import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import path from 'path';
import { logger } from '../utils/logger';

// Client instances
let dataConnectorClient: any = null;
let unifiedProcessorClient: any = null;
let embeddingsClient: any = null;
let relationGraphClient: any = null;
let mcpClient: any = null;
let featureToggleClient: any = null;
let authClient: any = null;

/**
 * Initialize all gRPC clients
 */
export async function initGrpcClients() {
    logger.info('Initializing gRPC clients for all downstream services');

    // Data Connector
    const connectorProto = loadProto('../data-connector/proto/connector.proto');
    dataConnectorClient = new connectorProto.confuse.connector.v1.DataConnector(
        process.env.DATA_CONNECTOR_GRPC_ADDR || 'data-connector:50052',
        grpc.credentials.createInsecure()
    );

    // Unified Processor
    const processorProto = loadProto('../unified-processor/proto/processor.proto');
    unifiedProcessorClient = new processorProto.confuse.processor.v1.UnifiedProcessor(
        process.env.UNIFIED_PROCESSOR_GRPC_ADDR || 'unified-processor:50053',
        grpc.credentials.createInsecure()
    );

    // Embeddings
    const embeddingsProto = loadProto('../embeddings-service/proto/embeddings.proto');
    embeddingsClient = new embeddingsProto.confuse.embeddings.v1.Embeddings(
        process.env.EMBEDDINGS_SERVICE_GRPC_ADDR || 'embeddings-service:50054',
        grpc.credentials.createInsecure()
    );

    // Relation Graph
    const graphProto = loadProto('../relation-graph/proto/graph.proto');
    relationGraphClient = new graphProto.confuse.graph.v1.RelationGraph(
        process.env.RELATION_GRAPH_GRPC_ADDR || 'relation-graph:50055',
        grpc.credentials.createInsecure()
    );

    // MCP Server
    const mcpProto = loadProto('../mcp-server/proto/mcp.proto');
    mcpClient = new mcpProto.confuse.mcp.v1.Mcp(
        process.env.MCP_SERVER_GRPC_ADDR || 'mcp-server:50056',
        grpc.credentials.createInsecure()
    );

    // Feature Toggle
    const toggleProto = loadProto('../feature-context-toggle/proto/toggle.proto');
    featureToggleClient = new toggleProto.confuse.toggle.v1.FeatureToggle(
        process.env.FEATURE_TOGGLE_GRPC_ADDR || 'feature-context-toggle:50057',
        grpc.credentials.createInsecure()
    );

    // Auth Middleware
    const authProto = loadProto('../auth-middleware/proto/auth.proto');
    authClient = new authProto.confuse.auth.v1.Auth(
        process.env.AUTH_MIDDLEWARE_GRPC_ADDR || 'auth-middleware:50058',
        grpc.credentials.createInsecure()
    );

    logger.info('All gRPC clients initialized successfully');
}

/**
 * Load proto file
 */
function loadProto(relativeProtoPath: string) {
    const protoPath = path.join(__dirname, relativeProtoPath);
    const packageDefinition = protoLoader.loadSync(protoPath, {
        keepCase: true,
        longs: String,
        enums: String,
        defaults: true,
        oneofs: true
    });
    return grpc.loadPackageDefinition(packageDefinition);
}

/**
 * Data Connector Client
 */
export const dataConnector = {
    async listSources(userId: string, type?: string, page = 1, pageSize = 50) {
        return promisify(dataConnectorClient.ListSources.bind(dataConnectorClient))({
            user_id: userId,
            type,
            page,
            page_size: pageSize
        });
    },

    async createSource(userId: string, type: string, name: string, uri: string, config: Record<string, string>) {
        return promisify(dataConnectorClient.CreateSource.bind(dataConnectorClient))({
            user_id: userId,
            type,
            name,
            uri,
            config
        });
    },

    async getSource(sourceId: string) {
        return promisify(dataConnectorClient.GetSource.bind(dataConnectorClient))({
            source_id: sourceId
        });
    },

    async ingestSource(sourceId: string, userId: string, path?: string) {
        return promisify(dataConnectorClient.IngestSource.bind(dataConnectorClient))({
            source_id: sourceId,
            user_id: userId,
            path
        });
    },

    async getJobStatus(jobId: string) {
        return promisify(dataConnectorClient.GetJobStatus.bind(dataConnectorClient))({
            job_id: jobId
        });
    }
};

/**
 * Unified Processor Client
 */
export const unifiedProcessor = {
    async processFile(fileId: string, filename: string, content: string, sourceType: string, metadata: Record<string, string>) {
        return promisify(unifiedProcessorClient.ProcessFile.bind(unifiedProcessorClient))({
            file_id: fileId,
            filename,
            content,
            source_type: sourceType,
            metadata
        });
    },

    async search(query: string, sourceId: string, limit = 10, filters: Record<string, string> = {}) {
        return promisify(unifiedProcessorClient.Search.bind(unifiedProcessorClient))({
            query,
            source_id: sourceId,
            limit,
            filters
        });
    },

    async chunkContent(content: string, contentType: string, options: Record<string, string> = {}) {
        return promisify(unifiedProcessorClient.ChunkContent.bind(unifiedProcessorClient))({
            content,
            content_type: contentType,
            options
        });
    }
};

/**
 * Embeddings Client
 */
export const embeddings = {
    async embed(text: string, model?: string, options: Record<string, string> = {}) {
        return promisify(embeddingsClient.Embed.bind(embeddingsClient))({
            text,
            model,
            options
        });
    },

    async batchEmbed(texts: string[], model?: string, options: Record<string, string> = {}) {
        return promisify(embeddingsClient.BatchEmbed.bind(embeddingsClient))({
            texts,
            model,
            options
        });
    },

    async getModelInfo(modelName?: string) {
        return promisify(embeddingsClient.GetModelInfo.bind(embeddingsClient))({
            model_name: modelName
        });
    }
};

/**
 * Relation Graph Client
 */
export const relationGraph = {
    async buildRelationships(sourceId: string, fileIds: string[], options: Record<string, string> = {}) {
        return promisify(relationGraphClient.BuildRelationships.bind(relationGraphClient))({
            source_id: sourceId,
            file_ids: fileIds,
            options
        });
    },

    async getEntity(entityId: string) {
        return promisify(relationGraphClient.GetEntity.bind(relationGraphClient))({
            entity_id: entityId
        });
    },

    async search(query: string, entityTypes: string[] = [], limit = 10) {
        return promisify(relationGraphClient.Search.bind(relationGraphClient))({
            query,
            entity_types: entityTypes,
            limit
        });
    }
};

/**
 * MCP Client
 */
export const mcp = {
    async listTools(category?: string) {
        return promisify(mcpClient.ListTools.bind(mcpClient))({
            category
        });
    },

    async callTool(toolId: string, parameters: Record<string, string>, userId: string, sessionId: string) {
        return promisify(mcpClient.CallTool.bind(mcpClient))({
            tool_id: toolId,
            parameters,
            user_id: userId,
            session_id: sessionId
        });
    }
};

/**
 * Feature Toggle Client
 */
export const featureToggle = {
    async getToggle(name: string) {
        return promisify(featureToggleClient.GetToggle.bind(featureToggleClient))({
            name
        });
    },

    async listToggles(category?: string) {
        return promisify(featureToggleClient.ListToggles.bind(featureToggleClient))({
            category
        });
    },

    async updateToggle(name: string, enabled: boolean) {
        return promisify(featureToggleClient.UpdateToggle.bind(featureToggleClient))({
            name,
            enabled
        });
    }
};

/**
 * Auth Client
 */
export const auth = {
    async validateToken(token: string) {
        return promisify(authClient.ValidateToken.bind(authClient))({
            token
        });
    },

    async getUser(userId: string) {
        return promisify(authClient.GetUser.bind(authClient))({
            user_id: userId
        });
    }
};

/**
 * Promisify gRPC callback-style calls
 */
function promisify(fn: Function) {
    return (request: any) => {
        return new Promise((resolve, reject) => {
            fn(request, (err: Error | null, response: any) => {
                if (err) {
                    logger.error('gRPC call failed', { error: err.message });
                    reject(err);
                } else {
                    resolve(response);
                }
            });
        });
    };
}

/**
 * Close all gRPC clients
 */
export async function closeGrpcClients() {
    logger.info('Closing all gRPC clients');

    // Close clients (if they have a close method)
    if (dataConnectorClient) dataConnectorClient.close();
    if (unifiedProcessorClient) unifiedProcessorClient.close();
    if (embeddingsClient) embeddingsClient.close();
    if (relationGraphClient) relationGraphClient.close();
    if (mcpClient) mcpClient.close();
    if (featureToggleClient) featureToggleClient.close();
    if (authClient) authClient.close();

    logger.info('All gRPC clients closed');
}
