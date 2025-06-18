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

# Remove actix-web-actors and actix (if not needed by others)
to_remove = ["actix-web-actors"]

# Check if actix is only needed by actix-web-actors
actix_needed_by_others = false
all_packages.each do |name, pkg|
  next if name == "actix-web-actors"
  if pkg.include?('"actix"')
    actix_needed_by_others = true
    break
  end
end

unless actix_needed_by_others
  to_remove << "actix"
  puts "actix is only needed by actix-web-actors, removing it too"
end

# Remove the packages
remaining = all_packages.keys - to_remove
test_content = header + remaining.map { |name| "[[package]]" + all_packages[name] }.join

File.write("Cargo.lock.no_actors", test_content)
puts "\nCreated Cargo.lock.no_actors"
puts "Removed: #{to_remove.join(', ')}"
puts "Remaining: #{remaining.length} packages"