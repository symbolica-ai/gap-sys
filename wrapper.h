// We must define EXPORT_INLINE as static inline for bindgen to work with libgap
#define EXPORT_INLINE static inline

// Include all of GAP's headers
#include <gap/libgap-api.h>
#include <gap/gap_all.h>

// Wrapper around macros
static inline int SYSGAP_Enter() {
    return GAP_Enter();
}

static inline void SYSGAP_Leave() {
    GAP_Leave();
}
