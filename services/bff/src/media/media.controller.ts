import {
  Controller,
  Get,
  HttpException,
  HttpStatus,
  MessageEvent,
  Param,
  Post,
  Query,
  Req,
  Sse,
  UploadedFile,
  UseInterceptors,
} from '@nestjs/common';
import { FileInterceptor } from '@nestjs/platform-express';
import { status as GrpcStatus } from '@grpc/grpc-js';
import type { Request } from 'express';
import { Observable } from 'rxjs';
import { toAssetDto, toStatusUpdateDto } from './dto';
import { MediaGrpcService } from './media-grpc.service';

const DEFAULT_PAGE_SIZE = 20;

@Controller('api')
export class MediaController {
  constructor(private readonly grpc: MediaGrpcService) {}

  @Get('assets')
  async listAssets(@Query('pageSize') pageSize?: string, @Query('pageToken') pageToken?: string) {
    const size = pageSize ? parseInt(pageSize, 10) : DEFAULT_PAGE_SIZE;
    const res = await this.grpc.listMedia(size, pageToken ?? '');
    return {
      assets: (res.assets ?? []).map(toAssetDto),
      nextPageToken: res.nextPageToken ?? '',
    };
  }

  @Get('assets/:id')
  async getAsset(@Param('id') id: string) {
    try {
      const res = await this.grpc.getMediaAsset(id);
      return toAssetDto(res);
    } catch (err) {
      const grpcErr = err as { code?: number };
      if (grpcErr.code === GrpcStatus.NOT_FOUND) {
        throw new HttpException('asset not found', HttpStatus.NOT_FOUND);
      }
      throw new HttpException('upstream error', HttpStatus.BAD_GATEWAY);
    }
  }

  // Buffers the file and re-posts it to core-service's REST endpoint —
  // simpler than true stream-proxying for the demo file sizes involved
  // (see plan's "Explicit Cuts").
  @Post('uploads')
  @UseInterceptors(FileInterceptor('file'))
  async upload(@UploadedFile() file?: Express.Multer.File) {
    if (!file) {
      throw new HttpException("missing 'file' field", HttpStatus.BAD_REQUEST);
    }

    const coreUrl = process.env.CORE_REST_URL ?? 'http://localhost:8080';
    const form = new FormData();
    form.append('file', new Blob([file.buffer], { type: file.mimetype }), file.originalname);

    const response = await fetch(`${coreUrl}/uploads`, { method: 'POST', body: form });
    const body = (await response.json()) as Record<string, unknown>;
    if (!response.ok) {
      throw new HttpException((body?.error as string) ?? 'upload failed', response.status);
    }
    return toAssetDto(body as any);
  }

  // Bridges core-service's gRPC server-stream into a browser-friendly
  // SSE stream — the browser never speaks gRPC directly.
  @Sse('events/asset/:id')
  streamAssetStatus(@Param('id') id: string, @Req() req: Request): Observable<MessageEvent> {
    return new Observable<MessageEvent>((subscriber) => {
      const call = this.grpc.streamMediaStatus(id);

      call.on('data', (update) => {
        subscriber.next({ data: toStatusUpdateDto(update) });
      });
      call.on('error', (err) => {
        subscriber.error(err);
      });
      call.on('end', () => {
        subscriber.complete();
      });

      const onClose = () => call.cancel();
      req.on('close', onClose);

      return () => {
        req.off('close', onClose);
        call.cancel();
      };
    });
  }
}
