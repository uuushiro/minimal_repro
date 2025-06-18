#!/usr/bin/env ruby

# Read the full Cargo.lock
content = File.read("Cargo.lock")
packages = content.split("[[package]]").reject(&:empty?)
header = packages.shift

# Find packages that might constrain actix-web versions
constraining_packages = []
packages.each do |pkg|
  lines = pkg.lines
  name_line = lines.find { |l| l.start_with?("name = ") }
  name = name_line.split('"')[1] if name_line
  
  # Check if this package depends on actix-web with a version constraint
  in_deps = false
  has_actix_constraint = false
  pkg.lines.each do |line|
    if line.strip == "dependencies = ["
      in_deps = true
    elsif line.strip == "]" && in_deps
      in_deps = false
    elsif in_deps && line.include?("actix-web")
      # Check if there's a version constraint
      if line.include?("version =") || line.include?("actix-web ")
        has_actix_constraint = true
        puts "#{name} has actix-web constraint: #{line.strip}"
      end
    end
  end
  
  if has_actix_constraint
    constraining_packages << name
  end
end

puts "\nPackages that depend on actix-web: #{constraining_packages.join(', ')}"

# Look for packages with specific version patterns
puts "\nLooking for packages with version constraints..."
packages.each do |pkg|
  if pkg.include?("4.3.1+deprecated") || pkg.include?("4.9.0") || pkg.include?("4.10") || pkg.include?("4.11")
    lines = pkg.lines
    name_line = lines.find { |l| l.start_with?("name = ") }
    version_line = lines.find { |l| l.start_with?("version = ") }
    if name_line && version_line
      name = name_line.split('"')[1]
      version = version_line.split('"')[1]
      puts "  #{name} = #{version}"
    end
  end
end

# Create a test with all actix-related packages and their dependencies
puts "\nCreating test with all actix ecosystem packages..."
actix_packages = []
packages.each do |pkg|
  lines = pkg.lines
  name_line = lines.find { |l| l.start_with?("name = ") }
  name = name_line.split('"')[1] if name_line
  
  if name && (name.start_with?("actix") || constraining_packages.include?(name) || 
              ["backend", "graphql", "sql-entities", "async-graphql", "async-graphql-actix-web",
               "sea-orm", "sqlx", "tokio", "serde", "futures-core", "futures-util"].include?(name))
    actix_packages << pkg
  end
end

minimal_content = header + actix_packages.map { |pkg| "[[package]]" + pkg }.join
File.write("Cargo.lock.actix_ecosystem", minimal_content)
puts "\nCreated Cargo.lock.actix_ecosystem with #{actix_packages.length} packages"