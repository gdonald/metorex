# Decimal numbers with as many digits as they are given, rather than as many
# as a float has room for.

require 'bigdecimal'

third = BigDecimal("1") / BigDecimal("3")
p(third.to_s)
p((BigDecimal("0.1") + BigDecimal("0.2")).to_s)
p((0.1 + 0.2 == 0.3))
p((BigDecimal("0.1") + BigDecimal("0.2") == BigDecimal("0.3")))

# A value is a sign, its significant digits, and the power of ten they sit
# against, which is what `split` names.
p(BigDecimal("-123.45").split)
p(BigDecimal("123.45").exponent)
p(BigDecimal("123.45").precision)

# The written form spells the number out in full when asked to.
p(BigDecimal("123.45").to_s)
p(BigDecimal("123.45").to_s("F"))
p(BigDecimal("1000010").to_s("5F"))

# Arithmetic carries the digits the caller asks for.
p(BigDecimal("2").sqrt(20).to_s)
p((BigDecimal("7") % BigDecimal("3")).to_s)
p(BigDecimal("7").divmod(BigDecimal("3")).map { |part| part.to_s })
p((BigDecimal("2") ** 10).to_s)

# Rounding takes a rule by name or by number.
p(BigDecimal("2.5").round(0, BigDecimal::ROUND_HALF_EVEN))
p(BigDecimal("2.5").round(0, :half_up))
p(BigDecimal("-1.5").floor)
p(BigDecimal("-1.5").ceil)
p(BigDecimal("1.987").truncate(2).to_s)

# The values that name themselves rather than a quantity.
p(BigDecimal("NaN").nan?)
p(BigDecimal("Infinity").infinite?)
p(BigDecimal("0").zero?)
p(BigDecimal("-3").sign == BigDecimal::SIGN_NEGATIVE_FINITE)

# A number too wide for a machine word is still one number.
p(BigDecimal("1234567890123456789012345679").remainder(BigDecimal("1")).to_s)
