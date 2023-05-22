// We must define EXPORT_INLINE as static inline for bindgen to work with libgap
#define EXPORT_INLINE static inline

// Include all of GAP's headers
#include </usr/local/gap/include/gap/libgap-api.h>
#include </usr/local/gap/include/gap/gap_all.h>

// Wrapper around macros
static inline void SYSGAP_Enter() {
    GAP_Enter();
}

static inline void SYSGAP_Leave() {
    GAP_Leave();
}
