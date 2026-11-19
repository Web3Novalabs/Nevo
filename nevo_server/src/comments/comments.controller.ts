import {
  Body,
  Controller,
  Delete,
  Get,
  HttpCode,
  HttpStatus,
  Param,
  ParseIntPipe,
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
import { CreateCommentDto } from './dto/create-comment.dto';
import { GetCommentsDto } from './dto/get-comments.dto';
import { UpdateCommentDto } from './dto/update-comment.dto';

interface AuthenticatedRequest extends Request {
  user: {
    walletAddress: string;
    [key: string]: unknown;
  };
}

@ApiTags('comments')
@Controller('pools/:id/comments')
export class CommentsController {
  constructor(private readonly commentsService: CommentsService) {}

  @Get()
  @ApiOperation({ summary: 'List comments for a pool (threaded, paginated)' })
  @ApiParam({ name: 'id', description: 'Pool ID', type: Number })
  @ApiQuery({ name: 'page', required: false, type: Number })
  @ApiQuery({ name: 'limit', required: false, type: Number })
  @ApiResponse({ status: 200, description: 'Paginated threaded comments' })
  @ApiResponse({ status: 404, description: 'Pool not found' })
  async findAll(
    @Param('id', ParseIntPipe) poolId: number,
    @Query() query: GetCommentsDto,
  ) {
    return this.commentsService.findByPool(poolId, query);
  }

  @Post()
  @UseGuards(JwtAuthGuard)
  @Throttle({ default: { limit: 5, ttl: 60_000 } })
  @ApiBearerAuth()
  @ApiOperation({ summary: 'Create a comment on a pool' })
  @ApiParam({ name: 'id', description: 'Pool ID', type: Number })
  @ApiResponse({ status: 201, description: 'Comment created' })
  @ApiResponse({ status: 400, description: 'Invalid input' })
  @ApiResponse({ status: 401, description: 'Unauthorized' })
  @ApiResponse({ status: 404, description: 'Pool or parent comment not found' })
  @ApiResponse({ status: 429, description: 'Too many requests' })
  async create(
    @Param('id', ParseIntPipe) poolId: number,
    @Body() dto: CreateCommentDto,
    @Req() req: AuthenticatedRequest,
  ) {
    return this.commentsService.create(poolId, req.user.walletAddress, dto);
  }

  @Patch(':commentId')
  @UseGuards(JwtAuthGuard)
  @ApiBearerAuth()
  @ApiOperation({ summary: 'Edit a comment (author or pool creator only)' })
  @ApiParam({ name: 'id', description: 'Pool ID', type: Number })
  @ApiParam({ name: 'commentId', description: 'Comment ID', type: Number })
  @ApiResponse({ status: 200, description: 'Comment updated' })
  @ApiResponse({ status: 400, description: 'Invalid input' })
  @ApiResponse({ status: 401, description: 'Unauthorized' })
  @ApiResponse({ status: 403, description: 'Forbidden' })
  @ApiResponse({ status: 404, description: 'Comment not found' })
  async update(
    @Param('id', ParseIntPipe) poolId: number,
    @Param('commentId', ParseIntPipe) commentId: number,
    @Body() dto: UpdateCommentDto,
    @Req() req: AuthenticatedRequest,
  ) {
    return this.commentsService.update(
      poolId,
      commentId,
      req.user.walletAddress,
      dto,
    );
  }

  @Delete(':commentId')
  @UseGuards(JwtAuthGuard)
  @HttpCode(HttpStatus.NO_CONTENT)
  @ApiBearerAuth()
  @ApiOperation({ summary: 'Delete a comment (author or pool creator only)' })
  @ApiParam({ name: 'id', description: 'Pool ID', type: Number })
  @ApiParam({ name: 'commentId', description: 'Comment ID', type: Number })
  @ApiResponse({ status: 204, description: 'Comment deleted' })
  @ApiResponse({ status: 401, description: 'Unauthorized' })
  @ApiResponse({ status: 403, description: 'Forbidden' })
  @ApiResponse({ status: 404, description: 'Comment not found' })
  async remove(
    @Param('id', ParseIntPipe) poolId: number,
    @Param('commentId', ParseIntPipe) commentId: number,
    @Req() req: AuthenticatedRequest,
  ) {
    await this.commentsService.remove(
      poolId,
      commentId,
      req.user.walletAddress,
    );
  }
}