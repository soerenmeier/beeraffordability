# Stage 1: Build the Craft CMS image with Composer installed
FROM ghcr.io/craftcms/image:8.4

# Override PHP configuration via environment variables
ENV PHP_UPLOAD_MAX_FILESIZE=28M
ENV PHP_POST_MAX_SIZE=28M
ENV PHP_MAX_EXECUTION_TIME=300

# Install Composer manually
USER root
RUN curl -sS https://getcomposer.org/installer | php -- --install-dir=/usr/local/bin --filename=composer

# Copy additional configuration files
COPY docker/nginx.conf /etc/nginx/nginx.conf
COPY docker/nginx.ini /etc/supervisord.d/nginx.ini

# Set the correct permissions for nginx files and directories
RUN mkdir -p /var/log/nginx && \
	chown -R appuser:appgroup /var/log/nginx && \
	chown -R appuser:appgroup /var/lib/nginx && \
	touch /run/nginx.pid && \
	chown -R appuser:appgroup /run/nginx.pid

# 3000
USER appuser

# Set the working directory to /app
WORKDIR /app

# Copy Composer files and application code
COPY --chown=appuser:appgroup . .

# Install dependencies with Composer
RUN composer install --no-dev --no-interaction --prefer-dist --optimize-autoloader
