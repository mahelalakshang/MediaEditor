import { Module } from '@nestjs/common';
import { MediaController } from './media.controller';
import { MediaGrpcService } from './media-grpc.service';

@Module({
  controllers: [MediaController],
  providers: [MediaGrpcService],
})
export class MediaModule {}
