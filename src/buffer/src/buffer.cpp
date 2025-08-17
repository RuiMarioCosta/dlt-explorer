#include "buffer.h"

#include <cstddef>

Buffer::Buffer(size_t capacity) { m_buffer.reserve(capacity); }

size_t Buffer::size() const { return m_buffer.size(); }
size_t Buffer::capacity() const { return m_buffer.capacity(); }
