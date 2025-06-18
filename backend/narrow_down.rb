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

puts "Current packages: #{all_packages.length}"

# List all packages by category
categories = {
  database: all_packages.keys.select { |k| k.include?("sea-") || k.include?("sqlx") || k.include?("postgres") || k.include?("mysql") || k.include?("sqlite") },
  crypto: all_packages.keys.select { |k| k.include?("sha") || k.include?("md5") || k.include?("hmac") || k.include?("digest") },
  encoding: all_packages.keys.select { |k| k.include?("base64") || k.include?("hex") || k.include?("percent") || k.include?("url") },
  async_extras: all_packages.keys.select { |k| k.match?(/^(crossbeam|parking_lot|thread|mio|signal)/) },
  proc_macro: all_packages.keys.select { |k| k.include?("proc-macro") || k.include?("darling") || k.include?("derive") },
  serialization: all_packages.keys.select { |k| k.include?("toml") || k.include?("json") && !k.include?("serde_json") },
  string_utils: all_packages.keys.select { |k| k.match?(/^(unicode|stringprep|idna|punycode)/) },
  misc_utils: all_packages.keys.select { |k| k.match?(/^(humantime|num-|ordered-|either|itertools|indexmap)/) }
}

# Report categories
categories.each do |cat, pkgs|
  puts "\n#{cat}: #{pkgs.length} packages"
  puts "  #{pkgs.take(10).join(', ')}#{'...' if pkgs.length > 10}"
end

# Test 1: Remove database packages
test_cases = [
  {
    name: "no_database",
    remove: categories[:database],
    desc: "Remove all database-related packages"
  },
  {
    name: "no_proc_macro", 
    remove: categories[:proc_macro],
    desc: "Remove proc-macro and derive packages"
  },
  {
    name: "no_misc",
    remove: categories[:crypto] + categories[:encoding] + categories[:string_utils] + categories[:misc_utils],
    desc: "Remove misc utility packages"
  }
]

test_cases.each do |test|
  remaining = all_packages.keys - test[:remove]
  test_content = header + remaining.map { |name| "[[package]]" + all_packages[name] }.join
  
  filename = "Cargo.lock.#{test[:name]}"
  File.write(filename, test_content)
  
  puts "\n#{test[:desc]}"
  puts "Created #{filename}"
  puts "Removed: #{test[:remove].length} packages"
  puts "Remaining: #{remaining.length} packages"
end