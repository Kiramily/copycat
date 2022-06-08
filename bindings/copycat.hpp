#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>


namespace CopyCat {

struct CopyFlags {
  uint32_t bits;

  explicit operator bool() const {
    return !!bits;
  }
  CopyFlags operator~() const {
    return {static_cast<decltype(bits)>(~bits)};
  }
  CopyFlags operator|(const CopyFlags& other) const {
    return {static_cast<decltype(bits)>(this->bits | other.bits)};
  }
  CopyFlags& operator|=(const CopyFlags& other) {
    *this = (*this | other);
    return *this;
  }
  CopyFlags operator&(const CopyFlags& other) const {
    return {static_cast<decltype(bits)>(this->bits & other.bits)};
  }
  CopyFlags& operator&=(const CopyFlags& other) {
    *this = (*this & other);
    return *this;
  }
  CopyFlags operator^(const CopyFlags& other) const {
    return {static_cast<decltype(bits)>(this->bits ^ other.bits)};
  }
  CopyFlags& operator^=(const CopyFlags& other) {
    *this = (*this ^ other);
    return *this;
  }
};
static const CopyFlags CopyFlags_NONE = CopyFlags{ /* .bits = */ (uint32_t)(1 << 0) };
static const CopyFlags CopyFlags_OVERWRITE = CopyFlags{ /* .bits = */ (uint32_t)(1 << 1) };
static const CopyFlags CopyFlags_RECURSIVE = CopyFlags{ /* .bits = */ (uint32_t)(1 << 2) };
static const CopyFlags CopyFlags_SKIP_EXISTING = CopyFlags{ /* .bits = */ (uint32_t)(1 << 3) };
static const CopyFlags CopyFlags_NO_OVERWRITE = CopyFlags{ /* .bits = */ (uint32_t)(1 << 4) };
static const CopyFlags CopyFlags_FOLLOW_SYMLINKS = CopyFlags{ /* .bits = */ (uint32_t)(1 << 5) };


extern "C" {

/// Copy a file or directory from one location to another.
/// If the source is a directory, the destination must be a directory as well.
/// If the source is a file, the destination must be a file as well.
///
/// # Panics
/// Idk tbh lmao
///
void cc_copy(const char *source, const char *destination, CopyFlags flags, size_t threads);

} // extern "C"

} // namespace CopyCat
