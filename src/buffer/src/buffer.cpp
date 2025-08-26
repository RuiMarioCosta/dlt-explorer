#include "buffer.h"

#include <cstddef>


Buffer::Buffer(size_t capacity) { m_buffer.reserve(capacity); }

size_t Buffer::size() const { return m_buffer.size(); }

size_t Buffer::capacity() const { return m_buffer.capacity(); }

Buffer::iterator Buffer::begin() { return m_buffer.begin(); }

Buffer::const_iterator Buffer::cbegin() const { return m_buffer.cbegin(); }

Buffer::const_iterator Buffer::iter() const {
  auto iter = m_buffer.begin();
  std::advance(iter, m_buffer.size());
  return iter;
}
