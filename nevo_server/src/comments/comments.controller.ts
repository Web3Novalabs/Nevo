import {
  Body,
  Controller,
  Delete,
  Get,
  HttpCode,
  HttpStatus,
  Param,
  ParseUUIDPipe,
  Patch,
  Post,
  Query,
  Req,
  UseGuards,
} from '@nestjs/common';
import {
  ApiBearerAuth,
  ApiOperation,
  ApiParam,
  ApiQuery,
  ApiResponse,
  ApiTags,
} from '@nestjs/swagger';
import { Throttle } from '@nestjs/throttler';
import { Request } from 'express';
import { JwtAuthGuard } from '../auth/jwt-auth.guard';
import { CommentsService } from './comments.service';
import {
  CreateCommentDto,
  GetCommentsDto,
  UpdateCommentDto,
} from './dto/create-comment.dto';

interface AuthenticatedRequest extends Request {
  user: {
    publicKey: string;
    [key: string]: unknown;
  };
}

@ApiTags('comments')
@Controller('pools/:id/comments')
export class CommentsController {
  constructor(private readonly commentsService: CommentsService) {}

  @Get()
  @ApiOperation({ summary: 'List comments for a pool (threaded, paginated)' })
  @ApiParam({ name: 'id', description: 'Pool ID', type: String })
  @ApiQuery({ name: 'page', required: false, type: String })
  @ApiQuery({ name: 'limit', required: false, type: Number })
  @ApiResponse({ status: 200, description: 'Paginated threaded comments' })
  @ApiResponse({ status: 404, description: 'Pool not found' })
  async findAll(
    @Param('id', ParseUUIDPipe) poolId: string,
    @Query() query: GetCommentsDto,
  ) {
    return this.commentsService.findForPool(poolId, query.page, query.limit);
  }

  @Post()
  @UseGuards(JwtAuthGuard)
  @Throttle({ default: { limit: 5, ttl: 60_000 } })
  @ApiBearerAuth()
  @ApiOperation({ summary: 'Create a comment on a pool' })
  @ApiParam({ name: 'id', description: 'Pool ID', type: String })
  @ApiResponse({ status: 201, description: 'Comment created' })
  @ApiResponse({ status: 400, description: 'Invalid input' })
  @ApiResponse({ status: 401, description: 'Unauthorized' })
  @ApiResponse({ status: 404, description: 'Pool or parent comment not found' })
  @ApiResponse({ status: 429, description: 'Too many requests' })
  async create(
    @Param('id', ParseUUIDPipe) poolId: string,
    @Body() dto: CreateCommentDto,
    @Req() req: AuthenticatedRequest,
  ) {
    return this.commentsService.create(poolId, req.user.publicKey, dto);
  }

  @Patch(':commentId')
  @UseGuards(JwtAuthGuard)
  @ApiBearerAuth()
  @ApiOperation({ summary: 'Edit a comment (author or pool creator only)' })
  @ApiParam({ name: 'id', description: 'Pool ID', type: String })
  @ApiParam({ name: 'commentId', description: 'Comment ID', type: String })
  @ApiResponse({ status: 200, description: 'Comment updated' })
  @ApiResponse({ status: 400, description: 'Invalid input' })
  @ApiResponse({ status: 401, description: 'Unauthorized' })
  @ApiResponse({ status: 403, description: 'Forbidden' })
  @ApiResponse({ status: 404, description: 'Comment not found' })
  async update(
    @Param('commentId', ParseUUIDPipe) commentId: string,
    @Body() dto: UpdateCommentDto,
    @Req() req: AuthenticatedRequest,
  ) {
    return this.commentsService.update(commentId, req.user.publicKey, dto);
  }

  @Delete(':commentId')
  @UseGuards(JwtAuthGuard)
  @HttpCode(HttpStatus.NO_CONTENT)
  @ApiBearerAuth()
  @ApiOperation({ summary: 'Delete a comment (author or pool creator only)' })
  @ApiParam({ name: 'id', description: 'Pool ID', type: String })
  @ApiParam({ name: 'commentId', description: 'Comment ID', type: String })
  @ApiResponse({ status: 204, description: 'Comment deleted' })
  @ApiResponse({ status: 401, description: 'Unauthorized' })
  @ApiResponse({ status: 403, description: 'Forbidden' })
  @ApiResponse({ status: 404, description: 'Comment not found' })
  async remove(
    @Param('commentId', ParseUUIDPipe) commentId: string,
    @Req() req: AuthenticatedRequest,
  ) {
    await this.commentsService.remove(commentId, req.user.publicKey);
  }
}
