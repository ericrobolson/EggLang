// This file was generated by wc-gen. Do not modify this file manually.

#include "../Point3.hpp"

Point3::Point3()
{
	y = 0;
	x = 0;
	z = 0;
}
Point3::Point3(const Point3 &other) : Point3()
{
	other.copy_to(*this);
}
Point3::~Point3()
{
}
void Point3::copy_to(Point3 &other) const
{
	other.y = y;
	other.x = x;
	other.z = z;
}
Point3 Point3::clone() const
{
	Point3 clone;
	copy_to(clone);
	return clone;
}
bool Point3::operator==(const Point3 &other) const
{
	return y == other.y && x == other.x && z == other.z;
}
bool Point3::operator!=(const Point3 &other) const
{
	return !(this == &other);
}
Point3 &Point3::operator=(const Point3 &other)
{
	if (this == &other)
	{
		return *this;
	}
	other.copy_to(*this);
	return *this;
}
