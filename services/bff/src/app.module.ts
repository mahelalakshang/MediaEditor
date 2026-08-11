import { Module } from '@nestjs/common';
import { AppController } from './app.controller';
import { MediaModule } from './media/media.module';

@Module({
  imports: [MediaModule],
  controllers: [AppController],
})
export class AppModule {}
