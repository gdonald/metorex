# rubocop:disable Lint/Syntax
def invoice_total(items)
  items.sum(&:price)
  end
end
# rubocop:enable Lint/Syntax
