// Memory operations: pointer-based load/store, array patterns
// Pointers are i32 in wasm32 — callers pass addresses into linear memory.

void store_i32(int* ptr, int val) { *ptr = val; }
int load_i32(int* ptr) { return *ptr; }

// Sum n i32 values starting at arr
int array_sum(int* arr, int n) {
    int sum = 0;
    while (n > 0) {
        sum += *arr;
        arr++;
        n--;
    }
    return sum;
}

// Find max in array of n i32 values
// Uses a running max variable (avoids select for comparison)
int array_max(int* arr, int n) {
    int max_val = *arr;
    arr++;
    n--;
    while (n > 0) {
        if (*arr > max_val) {
            max_val = *arr;
        }
        arr++;
        n--;
    }
    return max_val;
}

// Dot product of two i32 arrays of length n
int dot_product(int* a, int* b, int n) {
    int result = 0;
    while (n > 0) {
        result += (*a) * (*b);
        a++;
        b++;
        n--;
    }
    return result;
}

// Reverse an array in-place
void array_reverse(int* arr, int n) {
    int left = 0;
    int right = n - 1;
    while (left < right) {
        int tmp = arr[left];
        arr[left] = arr[right];
        arr[right] = tmp;
        left++;
        right--;
    }
}

// Bubble sort an array in-place
void bubble_sort(int* arr, int n) {
    int i = 0;
    while (i < n) {
        int j = 0;
        while (j < n - 1 - i) {
            if (arr[j] > arr[j + 1]) {
                int tmp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = tmp;
            }
            j++;
        }
        i++;
    }
}
