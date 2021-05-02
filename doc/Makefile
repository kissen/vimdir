RONN_ENVIRONMENT = --manual="vimdir(1)"

all: vidir.7 vidir.7.html

vidir.7: vidir.7.ronn
	ronn $(RONN_ENVIRONMENT) --roff $<

vidir.7.html: vidir.7.ronn
	ronn $(RONN_ENVIRONMENT) --html $<

clean:
	rm -f vidir.7 vidir.7.html

.PHONY: all clean
