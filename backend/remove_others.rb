#!/usr/bin/env ruby

# Read current Cargo.lock
content = File.read("Cargo.lock")
packages = content.split("[[package]]").reject(&:empty?)
header = packages.shift

# Parse all packages
all_packages = {}
packages.each do |pkg|
  lines = pkg.lines
  name_line = lines.find { |l| l.start_with?("name = ") }
  name = name_line.split('"')[1] if name_line
  all_packages[name] = pkg
end

# Remove specific suspect packages
suspects = ["chrono", "time", "sea-query", "rust_decimal", "bigdecimal", "uuid", "js-sys", "wasm-bindgen"]

remaining = all_packages.keys - suspects
test_content = header + remaining.map { |name| "[[package]]" + all_packages[name] }.join

File.write("Cargo.lock.no_suspects", test_content)
puts "Removed suspect packages: #{suspects.join(', ')}"
puts "Remaining: #{remaining.length} packages"

# Also create version without async-trait and other async helpers
async_suspects = ["async-trait", "async-stream", "tracing"]
remaining2 = all_packages.keys - async_suspects
test_content2 = header + remaining2.map { |name| "[[package]]" + all_packages[name] }.join

File.write("Cargo.lock.no_async_extras", test_content2)
puts "\nAlso created version without async extras"
puts "Removed: #{async_suspects.join(', ')}"
puts "Remaining: #{remaining2.length} packages"