# Text RubyGems writes out, with what a terminal would act on taken out.

module Gem
  module Text
    # Control characters replaced with a dot, so text a server sent cannot
    # move the cursor, retitle the window, or ring the bell. The C1 range is
    # only matched in valid UTF-8, where it cannot split a character.
    def clean_text(text)
      text = text.gsub(/[\000-\b\v-\f\016-\037\177]/, ".")
      if text.encoding == Encoding::UTF_8 && text.valid_encoding?
        text = text.gsub(/[\u0080-\u009f]/, ".")
      end
      text
    end
  end
end
