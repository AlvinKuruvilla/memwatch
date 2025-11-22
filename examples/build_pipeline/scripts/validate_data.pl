#!/usr/bin/env perl
#
# Data Validation Script for Build Pipeline Example
#
# Demonstrates Perl in the build pipeline for data validation,
# log parsing, and report generation.
#
# Usage:
#   perl validate_data.pl --size small --output dist

use strict;
use warnings;
use Getopt::Long;
use File::Path qw(make_path);
use File::Basename;
use Digest::SHA qw(sha256_hex);

# Problem size configurations
my %SIZES = (
    'small' => {
        'items'       => 1000,
        'description' => 'Quick validation (~20 MB memory)',
    },
    'medium' => {
        'items'       => 10000,
        'description' => 'Moderate validation (~100 MB memory)',
    },
    'large' => {
        'items'       => 50000,
        'description' => 'Intensive validation (~500 MB memory)',
    },
);

# Parse command line arguments
my $size   = 'small';
my $output = 'dist';
my $help   = 0;

GetOptions(
    'size=s'   => \$size,
    'output=s' => \$output,
    'help'     => \$help,
) or die("Error in command line arguments\n");

if ($help) {
    print_usage();
    exit(0);
}

unless (exists $SIZES{$size}) {
    die "Invalid size '$size'. Must be: small, medium, or large\n";
}

# Main execution
main();

sub main {
    my $config = $SIZES{$size};

    print "=" x 50, "\n";
    print "Data Validation for Build Pipeline Example\n";
    print "=" x 50, "\n";
    print "Size: $size\n";
    print "Description: $config->{description}\n";
    print "Items: $config->{items}\n";
    print "\n";

    # Step 1: Generate test data
    print "Step 1: Generating test data\n";
    my @data = generate_test_data($config->{items});
    print "  ✓ Generated ", scalar(@data), " items\n";
    print "\n";

    # Step 2: Validate data
    print "Step 2: Validating data integrity\n";
    my $validation_result = validate_data(\@data);
    print "  ✓ Validation ", $validation_result ? "passed" : "failed", "\n";
    print "\n";

    # Step 3: Compute statistics
    print "Step 3: Computing statistics\n";
    my $stats = compute_statistics(\@data);
    print_statistics($stats);
    print "\n";

    # Step 4: Generate report
    print "Step 4: Generating validation report\n";
    make_path($output) unless -d $output;
    my $report_file = "$output/validation_report.txt";
    generate_report($report_file, \@data, $stats);
    print "  ✓ Report written to $report_file\n";
    print "\n";

    print "=" x 50, "\n";
    print "Validation Complete\n";
    print "=" x 50, "\n";
}

sub generate_test_data {
    my ($count) = @_;
    my @data;

    print "  Generating $count test items...\n";

    for my $i (0 .. $count - 1) {
        my %item = (
            id        => $i,
            value     => int(rand(1000000)),
            timestamp => time() + $i,
            hash      => '',
        );

        # Compute hash (memory intensive for large datasets)
        my $data_str = join(':', $item{id}, $item{value}, $item{timestamp});
        $item{hash} = sha256_hex($data_str);

        push @data, \%item;

        # Print progress for large datasets
        if ($count > 10000 && $i % 10000 == 0 && $i > 0) {
            printf "    Progress: %d/%d (%.1f%%)\n",
              $i, $count, ($i / $count) * 100;
        }
    }

    return @data;
}

sub validate_data {
    my ($data_ref) = @_;
    my @data = @$data_ref;

    print "  Validating ", scalar(@data), " items...\n";

    # Check for duplicates (memory intensive - creates hash table)
    my %seen_ids;
    my %seen_hashes;
    my $duplicates = 0;

    for my $item (@data) {
        # Validate structure
        return 0 unless exists $item->{id};
        return 0 unless exists $item->{value};
        return 0 unless exists $item->{hash};

        # Check duplicates
        if (exists $seen_ids{ $item->{id} }) {
            $duplicates++;
        }
        $seen_ids{ $item->{id} } = 1;

        if (exists $seen_hashes{ $item->{hash} }) {
            $duplicates++;
        }
        $seen_hashes{ $item->{hash} } = 1;
    }

    if ($duplicates > 0) {
        print "  ⚠ Warning: Found $duplicates duplicate(s)\n";
    }

    return 1;
}

sub compute_statistics {
    my ($data_ref) = @_;
    my @data = @$data_ref;

    print "  Computing statistics for ", scalar(@data), " items...\n";

    # Extract values
    my @values = map { $_->{value} } @data;

    # Compute statistics
    my $count = scalar(@values);
    my $sum   = 0;
    my $min   = $values[0];
    my $max   = $values[0];

    for my $val (@values) {
        $sum += $val;
        $min = $val if $val < $min;
        $max = $val if $val > $max;
    }

    my $mean = $sum / $count;

    # Compute standard deviation (memory intensive - requires two passes)
    my $variance_sum = 0;
    for my $val (@values) {
        $variance_sum += ($val - $mean)**2;
    }
    my $stddev = sqrt($variance_sum / $count);

    return {
        count  => $count,
        sum    => $sum,
        min    => $min,
        max    => $max,
        mean   => $mean,
        stddev => $stddev,
    };
}

sub print_statistics {
    my ($stats) = @_;

    print "  Statistics:\n";
    print "    Count:  $stats->{count}\n";
    print "    Sum:    $stats->{sum}\n";
    print "    Min:    $stats->{min}\n";
    print "    Max:    $stats->{max}\n";
    printf "    Mean:   %.2f\n", $stats->{mean};
    printf "    StdDev: %.2f\n", $stats->{stddev};
}

sub generate_report {
    my ($filename, $data_ref, $stats) = @_;

    open my $fh, '>', $filename
      or die "Failed to open report file: $!\n";

    print $fh "=" x 60, "\n";
    print $fh "Data Validation Report\n";
    print $fh "=" x 60, "\n";
    print $fh "Generated: ", scalar(localtime()), "\n";
    print $fh "Size: $size\n";
    print $fh "\n";

    print $fh "Statistics:\n";
    print $fh "-" x 60, "\n";
    print $fh sprintf("  Items:       %d\n",  $stats->{count});
    print $fh sprintf("  Sum:         %d\n",  $stats->{sum});
    print $fh sprintf("  Min:         %d\n",  $stats->{min});
    print $fh sprintf("  Max:         %d\n",  $stats->{max});
    print $fh sprintf("  Mean:        %.2f\n", $stats->{mean});
    print $fh sprintf("  Std Dev:     %.2f\n", $stats->{stddev});
    print $fh "\n";

    print $fh "Sample Data (first 10 items):\n";
    print $fh "-" x 60, "\n";
    for my $i (0 .. 9) {
        last if $i >= scalar(@$data_ref);
        my $item = $data_ref->[$i];
        print $fh sprintf("  [%d] value=%d hash=%s\n",
                         $item->{id},
                         $item->{value},
                         substr($item->{hash}, 0, 16));
    }

    close $fh;
}

sub print_usage {
    print "Usage: perl validate_data.pl [OPTIONS]\n";
    print "\n";
    print "Options:\n";
    print "  --size SIZE    Problem size: small, medium, large (default: small)\n";
    print "  --output DIR   Output directory (default: dist)\n";
    print "  --help         Show this help message\n";
    print "\n";
}
