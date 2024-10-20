#include "complex_maths.h"

complex CMPLX(double real, double imag) {
	return complex { real, imag }
}

double creal(complex c) {
	return c.real
}

double cimag(complex c) {
	return c.imag
}

