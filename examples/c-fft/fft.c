// fft.c — 4096-point radix-2 Cooley-Tukey DIT FFT
//
// Freestanding C for wasm32-unknown-unknown (no libc, no libm).
//
// Twiddle factors are computed at runtime using:
//   - Taylor series for sin/cos at the tiny base angle δ = 2π/N
//   - A two-term recurrence to fill all N/2 entries from that base
//
// Magnitudes are computed via __builtin_sqrtf(), which compiles to the
// Wasm f32.sqrt instruction — no libm import needed.

#define N       4096
#define N_HALF  2048
#define LOG2_N  12
#define M_PI    3.14159265358979323846f

// ── Static global buffers (BSS) ────────────────────────────────────────────
// Combined size: 32768 + 8192 + 8192 + 8192 = 57344 bytes — fits in 64KB

static float g_signal[N * 2];       // interleaved complex input/output
static float g_twiddle_re[N_HALF];  // W_N^k real parts: cos(-2πk/N)
static float g_twiddle_im[N_HALF];  // W_N^k imag parts: -sin(2πk/N)
static float g_magnitude[N_HALF];   // power spectrum output

// ── Bit-reversal ────────────────────────────────────────────────────────────

static int bit_rev(int x) {
    // Reverse LOG2_N=12 bits
    int r = 0;
    for (int i = 0; i < LOG2_N; i++) {
        r = (r << 1) | (x & 1);
        x >>= 1;
    }
    return r;
}

static void apply_bit_reversal(void) {
    for (int i = 0; i < N; i++) {
        int j = bit_rev(i);
        if (j > i) {
            float tmp_re       = g_signal[2*i];
            float tmp_im       = g_signal[2*i+1];
            g_signal[2*i]      = g_signal[2*j];
            g_signal[2*i+1]    = g_signal[2*j+1];
            g_signal[2*j]      = tmp_re;
            g_signal[2*j+1]    = tmp_im;
        }
    }
}

// ── Twiddle computation ─────────────────────────────────────────────────────

// Taylor series sin/cos for small angle x (|x| < 0.002 for N=4096)
// sin(x) ≈ x - x³/6 + x⁵/120
// cos(x) ≈ 1 - x²/2 + x⁴/24

static float taylor_sin(float x) {
    float x2 = x * x;
    float x3 = x2 * x;
    float x5 = x3 * x2;
    return x - x3 * 0.16666667f + x5 * 0.00833333f;
}

static float taylor_cos(float x) {
    float x2 = x * x;
    float x4 = x2 * x2;
    return 1.0f - x2 * 0.5f + x4 * 0.04166667f;
}

// Fill twiddle table using complex recurrence:
//   (cos((k+1)δ), -sin((k+1)δ)) = (cos(kδ), -sin(kδ)) × (cos(δ), -sin(δ))
// which is a rotation by -δ each step.

static void compute_twiddles(void) {
    float delta = 2.0f * M_PI / (float)N;  // δ = 2π/4096 ≈ 0.001534

    float cd = taylor_cos(delta);   // cos(δ)
    float sd = taylor_sin(delta);   // sin(δ)

    // Seed: W_N^0 = (1, 0)
    float cr = 1.0f;
    float ci = 0.0f;  // ci tracks -sin(kδ), starts at 0

    g_twiddle_re[0] = 1.0f;
    g_twiddle_im[0] = 0.0f;

    for (int k = 1; k < N_HALF; k++) {
        // Rotation: new_cr = cr*cd - ci*(-sd) = cr*cd + ci*sd
        //           new_ci = ci*cd - cr*sd
        float new_cr = cr*cd + ci*sd;
        float new_ci = ci*cd - cr*sd;
        cr = new_cr;
        ci = new_ci;
        g_twiddle_re[k] = cr;
        g_twiddle_im[k] = ci;
    }
}

// ── DIT butterfly pass ──────────────────────────────────────────────────────

static void fft_dit(void) {
    for (int len = 2; len <= N; len <<= 1) {
        int half     = len >> 1;
        int tw_step  = N_HALF / half;   // stride into twiddle table
        for (int i = 0; i < N; i += len) {
            for (int j = 0; j < half; j++) {
                int    tw_idx  = j * tw_step;
                float  wr      = g_twiddle_re[tw_idx];
                float  wi      = g_twiddle_im[tw_idx];
                int    u       = i + j;
                int    v       = u + half;
                float  ur      = g_signal[2*u];
                float  ui_val  = g_signal[2*u+1];
                float  vr      = g_signal[2*v];
                float  vi_val  = g_signal[2*v+1];
                float  tr      = wr*vr - wi*vi_val;
                float  ti      = wr*vi_val + wi*vr;
                g_signal[2*u]   = ur + tr;
                g_signal[2*u+1] = ui_val + ti;
                g_signal[2*v]   = ur - tr;
                g_signal[2*v+1] = ui_val - ti;
            }
        }
    }
}

// ── Magnitude ───────────────────────────────────────────────────────────────

static void compute_magnitude(void) {
    for (int k = 0; k < N_HALF; k++) {
        float re = g_signal[2*k];
        float im = g_signal[2*k+1];
        float power = re*re + im*im;
        g_magnitude[k] = __builtin_sqrtf(power);
    }
}

// ── Exported API ─────────────────────────────────────────────────────────────

void fft_init(int n) {
    (void)n;   // reserved for future variable-N support
    compute_twiddles();
}

int fft_get_input_ptr(void) {
    return (int)(long)g_signal;
}

void fft_compute(int n) {
    (void)n;
    apply_bit_reversal();
    fft_dit();
    compute_magnitude();
}

int fft_get_output_ptr(void) {
    return (int)(long)g_magnitude;
}
