FROM busybox
RUN mkdir -p /dir1/dir2 && touch /dir1/file1 && touch /dir1/dir2/file2
RUN rm -rf /dir1/file1 && rm -rf /dir1/dir2 && mkdir /dir1/dir2 && touch /dir1/dir2/file3