#ifndef __COMPLEX_MATHS__
#define __COMPLEX_MATHS__

typedef struct {
	.real = double,
	.imag = double 
} complex;

complex CMPLX(double real, double imag);

double creal(complex c);

double cimag(complex c);

#endif

