#!/usr/bin/env ruby

# Read current Cargo.lock
content = File.read("Cargo.lock")
packages = content.split("[[package]]").reject(&:empty?)
header = packages.shift

# Parse all packages with their dependencies
all_packages = {}
packages.each do |pkg|
  lines = pkg.lines
  name_line = lines.find { |l| l.start_with?("name = ") }
  name = name_line.split('"')[1] if name_line
  
  deps = []
  in_deps = false
  pkg.lines.each do |line|
    if line.strip == "dependencies = ["
      in_deps = true
    elsif line.strip == "]" && in_deps
      in_deps = false
    elsif in_deps && line.include?('"')
      dep_name = line.strip.split('"')[1]
      deps << dep_name if dep_name
    end
  end
  
  all_packages[name] = { content: pkg, deps: deps }
end

puts "Total packages: #{all_packages.length}"

# Essential packages that MUST be kept
essential = [
  "backend", "graphql", "sql-entities",
  "actix-web", "async-graphql", "async-graphql-actix-web"
]

# Recursively find all dependencies
require 'set'
def get_all_deps(all_packages, name, visited = Set.new)
  return visited if visited.include?(name)
  return visited unless all_packages[name]
  
  visited.add(name)
  all_packages[name][:deps].each do |dep|
    get_all_deps(all_packages, dep, visited)
  end
  visited
end

# Get all required packages
required = Set.new
essential.each do |pkg|
  required.merge(get_all_deps(all_packages, pkg))
end

puts "Required packages (including dependencies): #{required.size}"

# Find packages that can be safely removed
safe_to_remove = all_packages.keys - required.to_a

# But we should not remove packages that are depended on by non-essential packages
# This ensures the lockfile remains valid
actually_safe = []
safe_to_remove.each do |pkg|
  # Check if any remaining package depends on this
  is_depended_on = false
  all_packages.each do |name, info|
    next if safe_to_remove.include?(name)
    if info[:deps].include?(pkg)
      is_depended_on = true
      break
    end
  end
  actually_safe << pkg unless is_depended_on
end

puts "Packages safe to remove: #{actually_safe.length}"

# Remove only truly safe packages
remaining_packages = all_packages.keys - actually_safe
test_content = header + remaining_packages.map { |name| "[[package]]" + all_packages[name][:content] }.join

File.write("Cargo.lock.safe", test_content)
puts "\nCreated Cargo.lock.safe with #{remaining_packages.length} packages"
puts "This should maintain a valid Cargo.lock structure"