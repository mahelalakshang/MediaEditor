import { Injectable, OnModuleInit } from '@nestjs/common';
import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import * as path from 'path';

export interface GrpcRendition {
  id: string;
  renditionType: string;
  path: string;
  width: number;
  height: number;
  score?: number;
  isSelected: boolean;
  timestampSeconds?: number;
}

export interface GrpcMediaAsset {
  id: string;
  originalFilename: string;
  mimeType: string;
  kind: string;
  status: string;
  createdAt: string;
  renditions: GrpcRendition[];
  originalPath: string;
}

export interface GrpcListMediaResponse {
  assets: GrpcMediaAsset[];
  nextPageToken: string;
}

export interface GrpcMediaStatusUpdate {
  assetId: string;
  status: string;
  message?: string;
  // proto-loader's `longs: String` option renders the wire int64 as a
  // JS string to avoid precision loss above 2^53 — not a number.
  updatedAtMs: string;
  progressHint?: string;
}

interface MediaServiceClient extends grpc.Client {
  getMediaAsset(
    request: { assetId: string },
    callback: (err: grpc.ServiceError | null, response: GrpcMediaAsset) => void,
  ): void;
  listMedia(
    request: { pageSize: number; pageToken: string },
    callback: (err: grpc.ServiceError | null, response: GrpcListMediaResponse) => void,
  ): void;
  streamMediaStatus(request: {
    assetId: string;
  }): grpc.ClientReadableStream<GrpcMediaStatusUpdate>;
}

// The BFF talks real gRPC to core-service (the service-to-service leg);
// see media.controller.ts for where that gets translated into REST/SSE
// for the browser, which cannot speak gRPC directly.
@Injectable()
export class MediaGrpcService implements OnModuleInit {
  private client!: MediaServiceClient;

  onModuleInit(): void {
    const protoPath =
      process.env.MEDIA_PROTO_PATH ?? path.resolve(process.cwd(), '../../proto/media.proto');

    const packageDefinition = protoLoader.loadSync(protoPath, {
      keepCase: false,
      longs: String,
      enums: String,
      defaults: true,
      oneofs: true,
    });
    const proto = grpc.loadPackageDefinition(packageDefinition) as any;

    const coreGrpcUrl = process.env.CORE_GRPC_URL ?? 'localhost:50051';
    this.client = new proto.media.v1.MediaService(
      coreGrpcUrl,
      grpc.credentials.createInsecure(),
    ) as MediaServiceClient;
  }

  getMediaAsset(assetId: string): Promise<GrpcMediaAsset> {
    return new Promise((resolve, reject) => {
      this.client.getMediaAsset({ assetId }, (err, response) => {
        if (err) reject(err);
        else resolve(response);
      });
    });
  }

  listMedia(pageSize: number, pageToken: string): Promise<GrpcListMediaResponse> {
    return new Promise((resolve, reject) => {
      this.client.listMedia({ pageSize, pageToken }, (err, response) => {
        if (err) reject(err);
        else resolve(response);
      });
    });
  }

  streamMediaStatus(assetId: string): grpc.ClientReadableStream<GrpcMediaStatusUpdate> {
    return this.client.streamMediaStatus({ assetId });
  }
}
